//! The Live client and related API types. Used for both real-time data and intraday historical.

use std::collections::HashMap;

use dbn::{
    decode::dbn::{AsyncMetadataDecoder, AsyncRecordDecoder},
    enums::{SType, Schema},
    record::SymbolMappingMsg,
    record_ref::RecordRef,
    Metadata,
};
use hex::ToHex;
use log::{debug, error, info};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
};
use typed_builder::TypedBuilder;

use crate::{validate_key, Error, Symbols, API_KEY_LENGTH};

/// The Live client. Used for subscribing to real-time and intraday historical market data.
///
/// Use [`LiveClient::builder()`](Client::builder) to get a type-safe builder for
/// initializing the required parameters for the client.
pub struct Client {
    key: String,
    dataset: String,
    send_ts_out: bool,
    connection: WriteHalf<TcpStream>,
    decoder: AsyncRecordDecoder<BufReader<ReadHalf<TcpStream>>>,
    session_id: String,
}

const BUCKET_ID_LENGTH: usize = 5;

impl Client {
    /// Returns a type-safe builder for setting the required parameters
    /// for initializing a [`LiveClient`](Client).
    pub fn builder() -> ClientBuilder<Unset, Unset> {
        ClientBuilder::default()
    }

    /// Creates a new client connected to a Live gateway.
    ///
    /// # Errors
    /// This function returns an error when `key` is invalid or its unable to connect
    /// and authenticate with the Live gateway.
    pub async fn connect(key: String, dataset: String, send_ts_out: bool) -> crate::Result<Self> {
        Self::connect_with_addr(Self::determine_gateway(&dataset), key, dataset, send_ts_out).await
    }

    /// Creates a new client connected to the Live gateway at `addr`. This is an advanced method and generally
    /// [`builder()`](Self::builder) or [`connect()`](Self::connect) should be used instead.
    ///
    /// # Errors
    /// This function returns an error when `key` is invalid or its unable
    /// to connect and authenticate with the Live gateway.
    pub async fn connect_with_addr(
        addr: impl ToSocketAddrs,
        key: String,
        dataset: String,
        send_ts_out: bool,
    ) -> crate::Result<Self> {
        let key = validate_key(key)?;
        let stream = TcpStream::connect(addr).await?;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        // Authenticate CRAM
        let session_id =
            Self::cram_challenge(&mut reader, &mut writer, &key, &dataset, send_ts_out).await?;

        Ok(Self {
            key,
            dataset,
            send_ts_out,
            connection: writer,
            decoder: AsyncRecordDecoder::new(reader),
            session_id,
        })
    }

    /// Returns the API key used by the instance of the client.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the dataset the client is configured for.
    pub fn dataset(&self) -> &str {
        &self.dataset
    }

    /// Returns an identifier for the current Live session.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Returns whether client is configured to request that the gateway send `ts_out`
    /// with each message.
    pub fn send_ts_out(&self) -> bool {
        self.send_ts_out
    }

    fn determine_gateway(dataset: &str) -> String {
        const DEFAULT_PORT: u16 = 13_000;

        let dataset_subdomain: String = dataset
            .chars()
            .map(|c| {
                if c == '.' {
                    '-'
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect();
        format!("{dataset_subdomain}.lsg.databento.com:{DEFAULT_PORT}")
    }

    async fn cram_challenge(
        reader: &mut BufReader<ReadHalf<TcpStream>>,
        writer: &mut WriteHalf<TcpStream>,
        key: &str,
        dataset: &str,
        send_ts_out: bool,
    ) -> crate::Result<String> {
        let mut greeting = String::new();
        // Greeting
        reader.read_line(&mut greeting).await?;
        greeting.pop(); // remove newline
        debug!("[{dataset}] Greeting: {greeting}");
        let mut response = String::new();
        // Challenge
        reader.read_line(&mut response).await?;
        response.pop(); // remove newline

        let challenge = if response.starts_with("cram=") {
            response.split_once('=').unwrap().1
        } else {
            error!("[{dataset}] No CRAM challenge in response from gateway: {response}");
            return Err(Error::internal(
                "no CRAM challenge in response from gateway",
            ));
        };

        // Parse challenge
        debug!("[{dataset}] Received CRAM challenge: {challenge}");

        // Construct reply
        let challenge_key = format!("{challenge}|{key}");
        let mut hasher = Sha256::new();
        hasher.update(challenge_key.as_bytes());
        let result = hasher.finalize();
        // Safe to slice because Databento only uses ASCII characters in API keys
        let bucket_id = &key[API_KEY_LENGTH - BUCKET_ID_LENGTH..];
        let encoded_response = result.encode_hex::<String>();
        let send_ts_out = send_ts_out as i32;
        let reply =
                format!("auth={encoded_response}-{bucket_id}|dataset={dataset}|encoding=dbn|ts_out={send_ts_out}|client=Rust {}\n", env!("CARGO_PKG_VERSION"));

        // Send CRAM reply
        debug!(
            "[{dataset}] Sending CRAM reply: {}",
            &reply[..reply.len() - 1]
        );
        writer.write_all(reply.as_bytes()).await.unwrap();

        response.clear();
        reader.read_line(&mut response).await?;

        debug!(
            "[{dataset}] Received auth response: {}",
            &response[..response.len() - 1]
        );

        response.pop(); // remove newline

        let mut auth_keys: HashMap<String, String> = response
            .split('|')
            .filter_map(|kvp| kvp.split_once('='))
            .map(|(key, val)| (key.to_owned(), val.to_owned()))
            .collect();
        if auth_keys.get_key_value("success").unwrap().1 != "1" {
            return Err(Error::Auth(
                auth_keys
                    .get("error")
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| "unknown".to_owned()),
            ));
        }
        debug!("[{dataset}] {response}");
        Ok(auth_keys.remove("session_id").unwrap_or_default())
    }

    /// Closes the connection with the gateway, ending the session and all subscriptions. Consumes
    /// the client.
    ///
    /// # Errors
    /// This function returns an error if the shutdown of the TCP stream is unsuccessful, this usually
    /// means the stream is no longer usable.
    pub async fn close(mut self) -> crate::Result<()> {
        Ok(self.connection.shutdown().await?)
    }

    /// Attempts to add a new subscription to the session. Note that
    /// an `Ok(())` result from this function does not necessarily indicate that
    /// the subscription succeeded, only that it was sent to the gateway.
    ///
    /// # Errors
    /// This function returns an error if it's unable to communicate with the gateway.
    pub async fn subscribe(&mut self, sub: &Subscription) -> crate::Result<()> {
        let Subscription {
            schema, stype_in, ..
        } = &sub;
        for sym_str in sub.symbols.to_chunked_api_string() {
            let args = format!("schema={schema}|stype_in={stype_in}|symbols={sym_str}");

            let sub_str = if let Some(start) = sub.start.as_ref() {
                format!("{args}|start={}\n", start.unix_timestamp_nanos())
            } else {
                format!("{args}\n")
            };

            debug!(
                "[{}] Subscribing: {}",
                self.dataset,
                &sub_str[..sub_str.len() - 1]
            );
            self.connection.write_all(sub_str.as_bytes()).await?;
        }
        Ok(())
    }

    /// Instructs the gateway to start sending data, starting the session. This method
    /// should only be called once on a given instance.
    ///
    /// Returns the DBN metadata associated with this session. This is primarily useful
    /// when saving the data to a file to replay it later.
    ///
    /// # Errors
    /// This function returns an error if it's unable to communicate with
    /// the gateway or there was an error decoding the DBN metadata.
    pub async fn start(&mut self) -> crate::Result<Metadata> {
        info!("[{}] Starting session", self.dataset);
        self.connection.write_all(b"start_session\n").await?;
        Ok(AsyncMetadataDecoder::new(self.decoder.get_mut())
            .decode()
            .await?)
    }

    /// Fetches the next record. This method should only be called after the session has
    /// been [started](Self::start).
    ///
    /// # Errors
    /// This function returns an error when it's unable to decode the next record
    /// or it's unable to read from the TCP stream.
    pub async fn next_record(&mut self) -> crate::Result<Option<RecordRef>> {
        Ok(self.decoder.decode_ref().await?)
    }
}

/// A subscription for real-time or intraday historical data.
#[derive(Debug, Clone, TypedBuilder)]
pub struct Subscription {
    /// The symbols of the instruments to subscribe to.
    #[builder(setter(into))]
    pub symbols: Symbols,
    /// The data record schema of data to subscribe to.
    pub schema: Schema,
    /// The symbology type of the symbols in [`symbols`](Self::symbols).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// If specified, requests available data since that time. When `None`,
    /// only real-time data is sent.
    ///
    /// Setting this field is not supported once the session has been started with
    /// [`LiveClient::start`](crate::LiveClient::start).
    #[builder(default, setter(strip_option))]
    pub start: Option<OffsetDateTime>,
}

#[doc(hidden)]
pub struct Unset;

/// A type-safe builder for the [`LiveClient`](Client). It will not allow you to call
/// [`Self::build()`] before setting the required fields:
/// - `key`
/// - `dataset`
pub struct ClientBuilder<AK, D> {
    key: AK,
    dataset: D,
    send_ts_out: bool,
}

impl Default for ClientBuilder<Unset, Unset> {
    fn default() -> Self {
        Self {
            key: Unset,
            dataset: Unset,
            send_ts_out: false,
        }
    }
}

impl<AK, D> ClientBuilder<AK, D> {
    /// Sets `ts_out`, which when enabled instructs the gateway to send a send timestamp
    /// after every record. These can be decoded with the special [`WithTsOut`](dbn::record::WithTsOut) type.
    pub fn send_ts_out(mut self, send_ts_out: bool) -> Self {
        self.send_ts_out = send_ts_out;
        self
    }
}

impl ClientBuilder<Unset, Unset> {
    /// Creates a new [`ClientBuilder`].
    pub fn new() -> Self {
        Self::default()
    }
}

impl<D> ClientBuilder<Unset, D> {
    /// Sets the API key.
    ///
    /// # Errors
    /// This function returns an error when the API key is invalid.
    pub fn key(self, key: impl ToString) -> crate::Result<ClientBuilder<String, D>> {
        Ok(ClientBuilder {
            key: crate::validate_key(key.to_string())?,
            dataset: self.dataset,
            send_ts_out: self.send_ts_out,
        })
    }

    /// Sets the API key reading it from the `DATABENTO_API_KEY` environment
    /// variable.
    ///
    /// # Errors
    /// This function returns an error when the environment variable is not set or the
    /// API key is invalid.
    pub fn key_from_env(self) -> crate::Result<ClientBuilder<String, D>> {
        let key = crate::key_from_env()?;
        self.key(key)
    }
}

impl<AK> ClientBuilder<AK, Unset> {
    /// Sets the dataset.
    pub fn dataset(self, dataset: impl ToString) -> ClientBuilder<AK, String> {
        ClientBuilder {
            key: self.key,
            dataset: dataset.to_string(),
            send_ts_out: self.send_ts_out,
        }
    }
}

impl ClientBuilder<String, String> {
    /// Initializes the client and attempts to connect to the gateway.
    ///
    /// # Errors
    /// This function returns an error when its unable
    /// to connect and authenticate with the Live gateway.
    pub async fn build(self) -> crate::Result<Client> {
        Client::connect(self.key, self.dataset, self.send_ts_out).await
    }
}

/// Manages the mapping between the instrument IDs included in each record and
/// a text symbology.
#[derive(Debug, Clone, Default)]
pub struct SymbolMap {
    symbol_map: HashMap<u32, String>,
}

impl SymbolMap {
    /// Creates a new `SymbolMap` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles updating the mappings (if required) for a generic record.
    ///
    /// # Errors
    /// This function returns an error when `record` contains a [`SymbolMappingMsg`] but
    /// it contains invalid UTF-8.
    pub fn on_record(&mut self, record: RecordRef) -> crate::Result<()> {
        if let Some(symbol_mapping) = record.get::<SymbolMappingMsg>() {
            self.on_symbol_mapping(symbol_mapping)
        } else {
            Ok(())
        }
    }

    /// Handles updating the mappings for a .
    ///
    /// # Errors
    /// This function returns an error when `symbol_mapping` contains invalid UTF-8.
    pub fn on_symbol_mapping(&mut self, symbol_mapping: &SymbolMappingMsg) -> crate::Result<()> {
        let stype_out_symbol = symbol_mapping.stype_out_symbol()?;
        info!(
            "Updated symbol mapping for {} to {}",
            symbol_mapping.hd.instrument_id, stype_out_symbol
        );
        self.symbol_map
            .insert(symbol_mapping.hd.instrument_id, stype_out_symbol.to_owned());
        Ok(())
    }

    /// Returns a reference to the mapping for the given instrument ID.
    pub fn get(&self, instrument_id: u32) -> Option<&String> {
        self.symbol_map.get(&instrument_id)
    }

    /// Returns a reference to the inner map.
    pub fn inner(&self) -> &HashMap<u32, String> {
        &self.symbol_map
    }

    /// Returns a mutable reference to the inner map.
    pub fn inner_mut(&mut self) -> &mut HashMap<u32, String> {
        &mut self.symbol_map
    }
}

impl std::ops::Index<u32> for SymbolMap {
    type Output = String;

    fn index(&self, instrument_id: u32) -> &Self::Output {
        self.get(instrument_id)
            .expect("symbol mapping for instrument ID")
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::c_char, fmt};

    use dbn::{
        encode::AsyncDbnMetadataEncoder,
        enums::rtype,
        publishers::Dataset,
        record::{HasRType, OhlcvMsg, RecordHeader, TradeMsg, WithTsOut},
        MetadataBuilder, UNDEF_TIMESTAMP,
    };
    use tokio::{net::TcpListener, sync::mpsc::UnboundedSender, task::JoinHandle};

    use super::*;

    struct MockLsgServer {
        dataset: String,
        send_ts_out: bool,
        listener: TcpListener,
        stream: Option<BufReader<TcpStream>>,
    }

    impl MockLsgServer {
        async fn new(dataset: String, send_ts_out: bool) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            Self {
                dataset,
                send_ts_out,
                listener,
                stream: None,
            }
        }

        async fn accept(&mut self) {
            self.stream = Some(BufReader::new(self.listener.accept().await.unwrap().0));
        }

        async fn authenticate(&mut self) {
            self.accept().await;
            self.send("lsg-test\n").await;
            self.send("cram=t7kNhwj4xqR0QYjzFKtBEG2ec2pXJ4FK\n").await;
            let auth_line = self.read_line().await;
            let auth_start = auth_line.find("auth=").unwrap() + 5;
            let auth_end = auth_line[auth_start..].find('|').unwrap();
            let auth = &auth_line[auth_start..auth_start + auth_end];
            let (auth, bucket) = auth.split_once('-').unwrap();
            assert!(
                auth.chars().all(|c| c.is_ascii_hexdigit()),
                "Expected '{auth}' to be composed of only hex characters"
            );
            assert_eq!(bucket, "iller");
            assert!(auth_line.contains(&format!("dataset={}", self.dataset)));
            assert!(auth_line.contains("encoding=dbn"));
            assert!(auth_line.contains(&format!("ts_out={}", if self.send_ts_out { 1 } else { 0 })));
            assert!(auth_line.contains(&format!("client=Rust {}", env!("CARGO_PKG_VERSION"))));
            self.send("success=1|session_id=5\n").await;
        }

        async fn subscribe(&mut self, subscription: Subscription) {
            let sub_line = self.read_line().await;
            assert!(sub_line.contains(&format!("symbols={}", subscription.symbols.to_api_string())));
            assert!(sub_line.contains(&format!("schema={}", subscription.schema)));
            assert!(sub_line.contains(&format!("stype_in={}", subscription.stype_in)));
            if let Some(start) = subscription.start {
                assert!(sub_line.contains(&format!("start={}", start.unix_timestamp_nanos())))
            }
        }

        async fn start(&mut self) {
            let start_line = self.read_line().await;
            assert_eq!(start_line, "start_session\n");
            let dataset = self.dataset.clone();
            let stream = self.stream();
            let mut encoder = AsyncDbnMetadataEncoder::new(stream);
            encoder
                .encode(
                    &MetadataBuilder::new()
                        .dataset(dataset)
                        .start(time::OffsetDateTime::now_utc().unix_timestamp_nanos() as u64)
                        .schema(None)
                        .stype_in(None)
                        .stype_out(SType::InstrumentId)
                        .build(),
                )
                .await
                .unwrap();
        }

        async fn send(&mut self, bytes: &str) {
            self.stream().write_all(bytes.as_bytes()).await.unwrap();
            info!("Sent: {}", &bytes[..bytes.len() - 1])
        }

        async fn send_record(&mut self, record: Box<dyn AsRef<[u8]> + Send>) {
            let bytes = (*record).as_ref();
            self.stream().write_all(bytes).await.unwrap()
        }

        async fn read_line(&mut self) -> String {
            let mut res = String::new();
            self.stream().read_line(&mut res).await.unwrap();
            info!("Read: {}", &res[..res.len() - 1]);
            res
        }

        fn stream(&mut self) -> &mut BufReader<TcpStream> {
            self.stream.as_mut().unwrap()
        }
    }

    struct Fixture {
        send: UnboundedSender<Event>,
        port: u16,
        task: JoinHandle<()>,
    }

    enum Event {
        Stop,
        Authenticate,
        Subscribe(Subscription),
        Start,
        SendRecord(Box<dyn AsRef<[u8]> + Send>),
    }

    impl fmt::Debug for Event {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Event::Stop => write!(f, "Stop"),
                Event::Authenticate => write!(f, "Authenticate"),
                Event::Subscribe(sub) => write!(f, "Subscribe({sub:?})"),
                Event::Start => write!(f, "Start"),
                Event::SendRecord(_) => write!(f, "SendRecord"),
            }
        }
    }

    impl Fixture {
        pub async fn new(dataset: String, send_ts_out: bool) -> Self {
            let (send, mut recv) = tokio::sync::mpsc::unbounded_channel();
            let mut mock = MockLsgServer::new(dataset, send_ts_out).await;
            let port = mock.listener.local_addr().unwrap().port();
            let task = tokio::task::spawn(async move {
                loop {
                    match recv.recv().await {
                        Some(Event::Authenticate) => mock.authenticate().await,
                        Some(Event::Subscribe(sub)) => mock.subscribe(sub).await,
                        Some(Event::Start) => mock.start().await,
                        Some(Event::SendRecord(rec)) => mock.send_record(rec).await,
                        Some(Event::Stop) | None => break,
                    }
                }
            });
            Self { task, port, send }
        }

        pub fn authenticate(&mut self) {
            self.send.send(Event::Authenticate).unwrap();
        }

        pub fn expect_subscribe(&mut self, subscription: Subscription) {
            self.send.send(Event::Subscribe(subscription)).unwrap();
        }

        pub fn start(&mut self) {
            self.send.send(Event::Start).unwrap();
        }

        pub fn send_record<R>(&mut self, record: R)
        where
            R: HasRType + AsRef<[u8]> + Clone + Send + 'static,
        {
            self.send
                .send(Event::SendRecord(Box::new(record.clone())))
                .unwrap();
        }

        pub async fn stop(self) {
            self.send.send(Event::Stop).unwrap();
            self.task.await.unwrap()
        }
    }

    async fn setup(dataset: Dataset, send_ts_out: bool) -> (Fixture, Client) {
        let _ = env_logger::try_init();
        let mut fixture = Fixture::new(dataset.as_str().to_owned(), send_ts_out).await;
        fixture.authenticate();
        let target = Client::connect_with_addr(
            format!("127.0.0.1:{}", fixture.port),
            "32-character-with-lots-of-filler".to_owned(),
            dataset.as_str().to_owned(),
            send_ts_out,
        )
        .await
        .unwrap();
        (fixture, target)
    }

    #[tokio::test]
    async fn test_subscribe() {
        let (mut fixture, mut client) = setup(Dataset::XnasItch, false).await;
        let subscription = Subscription::builder()
            .symbols(vec!["MSFT", "TSLA", "QQQ"])
            .schema(Schema::Ohlcv1M)
            .stype_in(SType::RawSymbol)
            .build();
        fixture.expect_subscribe(subscription.clone());
        client.subscribe(&subscription).await.unwrap();
        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_subscription_chunking() {
        const SYMBOL: &str = "TEST";
        const SYMBOL_COUNT: usize = 1000;
        let (mut fixture, mut client) = setup(Dataset::XnasItch, false).await;
        let sub_base = Subscription::builder()
            .schema(Schema::Ohlcv1M)
            .stype_in(SType::RawSymbol);
        let subscription = sub_base.clone().symbols(vec![SYMBOL; SYMBOL_COUNT]).build();
        client.subscribe(&subscription).await.unwrap();
        let mut i = 0;
        while i < SYMBOL_COUNT {
            let chunk_size = 128.min(SYMBOL_COUNT - i);
            fixture.expect_subscribe(sub_base.clone().symbols(vec![SYMBOL; chunk_size]).build());
            i += chunk_size;
        }
        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_next_record() {
        const REC: OhlcvMsg = OhlcvMsg {
            hd: RecordHeader::new::<OhlcvMsg>(rtype::OHLCV_1M, 1, 2, 3),
            open: 1,
            high: 2,
            low: 3,
            close: 4,
            volume: 5,
        };
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, false).await;
        fixture.start();
        let metadata = client.start().await.unwrap();
        assert_eq!(metadata.version, 1);
        assert!(metadata.schema.is_none());
        assert_eq!(metadata.dataset, Dataset::GlbxMdp3.as_str());
        fixture.send_record(REC);
        let rec = client.next_record().await.unwrap().unwrap();
        assert_eq!(*rec.get::<OhlcvMsg>().unwrap(), REC);
        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_next_record_with_ts_out() {
        let expected = WithTsOut::new(
            TradeMsg {
                hd: RecordHeader::new::<TradeMsg>(rtype::MBP_0, 1, 2, 3),
                price: 1,
                size: 2,
                action: b'A' as c_char,
                side: b'A' as c_char,
                flags: 0,
                depth: 1,
                ts_recv: 0,
                ts_in_delta: 0,
                sequence: 2,
            },
            time::OffsetDateTime::now_utc().unix_timestamp_nanos() as u64,
        );
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, true).await;
        fixture.start();
        let metadata = client.start().await.unwrap();
        assert_eq!(metadata.version, 1);
        assert!(metadata.schema.is_none());
        assert_eq!(metadata.dataset, Dataset::GlbxMdp3.as_str());
        fixture.send_record(expected.clone());
        let rec = client.next_record().await.unwrap().unwrap();
        assert_eq!(*rec.get::<WithTsOut<TradeMsg>>().unwrap(), expected);
        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_symbol_mapping() -> crate::Result<()> {
        let mut target = SymbolMap::new();
        let (mut fixture, mut client) = setup(Dataset::XnasItch, true).await;
        fixture.start();
        client.start().await?;
        fixture.send_record(SymbolMappingMsg::new(
            1,
            2,
            "",
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        fixture.send_record(SymbolMappingMsg::new(
            2,
            2,
            "",
            "TSLA",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        fixture.send_record(SymbolMappingMsg::new(
            3,
            2,
            "",
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        target.on_record(client.next_record().await?.unwrap())?;
        target.on_record(client.next_record().await?.unwrap())?;
        target.on_record(client.next_record().await?.unwrap())?;
        assert_eq!(
            *target.inner(),
            HashMap::from([
                (1, "AAPL".to_owned()),
                (2, "TSLA".to_owned()),
                (3, "MSFT".to_owned())
            ])
        );
        fixture.send_record(SymbolMappingMsg::new(
            10,
            2,
            "",
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        target.on_symbol_mapping(
            client
                .next_record()
                .await?
                .unwrap()
                .get::<SymbolMappingMsg>()
                .unwrap(),
        )?;
        assert_eq!(target[10], "AAPL");
        fixture.send_record(SymbolMappingMsg::new(
            9,
            2,
            "",
            "MSFT",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        target.on_record(client.next_record().await?.unwrap())?;
        assert_eq!(target[9], "MSFT");

        // client.next_record().await.unwrap().unwrap();
        fixture.stop().await;
        Ok(())
    }

    #[tokio::test]
    async fn test_close() {
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, true).await;
        fixture.start();
        client.start().await.unwrap();
        client.close().await.unwrap();
        fixture.stop().await;
    }
}
