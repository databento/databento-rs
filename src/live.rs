//! The Live client and related API types. Used for both real-time data and intraday historical.

use std::{collections::HashMap, fmt};

use dbn::{
    compat::SymbolMappingMsgV1,
    decode::dbn::{AsyncMetadataDecoder, AsyncRecordDecoder},
    Metadata, PitSymbolMap, RecordRef, SType, Schema, SymbolMappingMsg, VersionUpgradePolicy,
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

use crate::{validate_key, ApiKey, Error, Symbols, API_KEY_LENGTH, BUCKET_ID_LENGTH};

/// The Live client. Used for subscribing to real-time and intraday historical market data.
///
/// Use [`LiveClient::builder()`](Client::builder) to get a type-safe builder for
/// initializing the required parameters for the client.
pub struct Client {
    key: ApiKey,
    dataset: String,
    send_ts_out: bool,
    upgrade_policy: VersionUpgradePolicy,
    connection: WriteHalf<TcpStream>,
    decoder: AsyncRecordDecoder<BufReader<ReadHalf<TcpStream>>>,
    session_id: String,
}

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
    pub async fn connect(
        key: String,
        dataset: String,
        send_ts_out: bool,
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        Self::connect_with_addr(
            Self::determine_gateway(&dataset),
            key,
            dataset,
            send_ts_out,
            upgrade_policy,
        )
        .await
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
        upgrade_policy: VersionUpgradePolicy,
    ) -> crate::Result<Self> {
        let key = validate_key(key)?;
        let stream = TcpStream::connect(addr).await?;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        // Authenticate CRAM
        let session_id =
            Self::cram_challenge(&mut reader, &mut writer, &key.0, &dataset, send_ts_out).await?;

        Ok(Self {
            key,
            dataset,
            send_ts_out,
            upgrade_policy,
            connection: writer,
            // Pass a placeholder DBN version and should never fail because DBN_VERSION
            // is a valid DBN version. Correct version set in `start()`.
            decoder: AsyncRecordDecoder::with_version(
                reader,
                dbn::DBN_VERSION,
                upgrade_policy,
                send_ts_out,
            )
            .unwrap(),
            session_id,
        })
    }

    /// Returns the API key used by the instance of the client.
    pub fn key(&self) -> &str {
        &self.key.0
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

    /// Returns the upgrade policy for decoding DBN from previous versions.
    pub fn upgrade_policy(&self) -> VersionUpgradePolicy {
        self.upgrade_policy
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
        // Lack of success key also indicates something went wrong
        if auth_keys.get("success").map(|v| v != "1").unwrap_or(true) {
            return Err(Error::Auth(
                auth_keys
                    .get("error")
                    .map(ToOwned::to_owned)
                    .unwrap_or_else(|| response),
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// [`tokio::select!`] statement and another branch completes first, the subscription
    /// may have been partially sent, resulting in the gateway rejecting the
    /// subscription, sending an error, and closing the connection.
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
    ///
    /// # Cancel safety
    /// This method is not cancellation safe. If this method is used in a
    /// [`tokio::select!`] statement and another branch completes first, the live
    /// gateway may only receive a partial message, resulting in it sending an error and
    /// closing the connection.
    pub async fn start(&mut self) -> crate::Result<Metadata> {
        info!("[{}] Starting session", self.dataset);
        self.connection.write_all(b"start_session\n").await?;
        let mut metadata = AsyncMetadataDecoder::new(self.decoder.get_mut())
            .decode()
            .await?;
        self.decoder.set_version(metadata.version)?;
        // Should match `send_ts_out` but set again here for safety
        self.decoder.set_ts_out(metadata.ts_out);
        metadata.upgrade(self.upgrade_policy);
        Ok(metadata)
    }

    /// Fetches the next record. This method should only be called after the session has
    /// been [started](Self::start).
    ///
    /// Returns `Ok(None)` if the gateway closed the connection and no more records
    /// can be read.
    ///
    /// # Errors
    /// This function returns an error when it's unable to decode the next record
    /// or it's unable to read from the TCP stream.
    ///
    /// # Cancel safety
    /// This method is cancel safe. It can be used within a [`tokio::select!`] statement
    /// without the potential for corrupting the input stream.
    pub async fn next_record(&mut self) -> crate::Result<Option<RecordRef>> {
        Ok(self.decoder.decode_ref().await?)
    }
}

/// A subscription for real-time or intraday historical data.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct Subscription {
    /// The symbols of the instruments to subscribe to.
    #[builder(setter(into))]
    pub symbols: Symbols,
    /// The data record schema of data to subscribe to.
    pub schema: Schema,
    /// The symbology type of the symbols in [`symbols`](Self::symbols).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// If specified, requests available data since that time (inclusive), based on
    /// [`ts_event`](dbn::RecordHeader::ts_event). When `None`, only real-time data is sent.
    ///
    /// Setting this field is not supported once the session has been started with
    /// [`LiveClient::start`](crate::LiveClient::start).
    #[builder(default, setter(strip_option))]
    pub start: Option<OffsetDateTime>,
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct Unset;

/// A type-safe builder for the [`LiveClient`](Client). It will not allow you to call
/// [`Self::build()`] before setting the required fields:
/// - `key`
/// - `dataset`
#[derive(Debug, Clone)]
pub struct ClientBuilder<AK, D> {
    key: AK,
    dataset: D,
    send_ts_out: bool,
    upgrade_policy: VersionUpgradePolicy,
}

impl Default for ClientBuilder<Unset, Unset> {
    fn default() -> Self {
        Self {
            key: Unset,
            dataset: Unset,
            send_ts_out: false,
            upgrade_policy: VersionUpgradePolicy::Upgrade,
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

    /// Sets `upgrade_policy`, which controls how to decode data from prior DBN
    /// versions. The current default is to decode them as-is.
    pub fn upgrade_policy(mut self, upgrade_policy: VersionUpgradePolicy) -> Self {
        self.upgrade_policy = upgrade_policy;
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
    pub fn key(self, key: impl ToString) -> crate::Result<ClientBuilder<ApiKey, D>> {
        Ok(ClientBuilder {
            key: crate::validate_key(key.to_string())?,
            dataset: self.dataset,
            send_ts_out: self.send_ts_out,
            upgrade_policy: self.upgrade_policy,
        })
    }

    /// Sets the API key reading it from the `DATABENTO_API_KEY` environment
    /// variable.
    ///
    /// # Errors
    /// This function returns an error when the environment variable is not set or the
    /// API key is invalid.
    pub fn key_from_env(self) -> crate::Result<ClientBuilder<ApiKey, D>> {
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
            upgrade_policy: self.upgrade_policy,
        }
    }
}

impl ClientBuilder<ApiKey, String> {
    /// Initializes the client and attempts to connect to the gateway.
    ///
    /// # Errors
    /// This function returns an error when its unable
    /// to connect and authenticate with the Live gateway.
    pub async fn build(self) -> crate::Result<Client> {
        Client::connect(
            self.key.0,
            self.dataset,
            self.send_ts_out,
            self.upgrade_policy,
        )
        .await
    }
}

/// Manages the mapping between the instrument IDs included in each record and
/// a text symbology.
#[derive(Debug, Clone, Default)]
#[deprecated(
    since = "0.5.0",
    note = "dbn::PitSymbolMap provides identical functionality and also works with historical data"
)]
pub struct SymbolMap {
    inner: PitSymbolMap,
}

#[allow(deprecated)]
impl SymbolMap {
    /// Creates a new `SymbolMap` instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles updating the mappings (if required) from a generic record.
    ///
    /// # Errors
    /// This function returns an error when `record` contains a [`SymbolMappingMsg`] but
    /// it contains invalid UTF-8.
    pub fn on_record(&mut self, record: RecordRef) -> crate::Result<()> {
        Ok(self.inner.on_record(record)?)
    }

    /// Handles updating the mappings from a [`SymbolMappingMsg`].
    ///
    /// # Errors
    /// This function returns an error when `symbol_mapping` contains invalid UTF-8.
    pub fn on_symbol_mapping(&mut self, symbol_mapping: &SymbolMappingMsg) -> crate::Result<()> {
        Ok(self.inner.on_symbol_mapping(symbol_mapping)?)
    }

    /// Handles updating the mappings from a [`SymbolMappingMsgV1`].
    ///
    /// # Errors
    /// This function returns an error when `symbol_mapping` contains invalid UTF-8.
    pub fn on_symbol_mapping_v1(
        &mut self,
        symbol_mapping: &SymbolMappingMsgV1,
    ) -> crate::Result<()> {
        Ok(self.inner.on_symbol_mapping(symbol_mapping)?)
    }

    /// Returns a reference to the mapping for the given instrument ID.
    pub fn get(&self, instrument_id: u32) -> Option<&String> {
        self.inner.get(instrument_id)
    }

    /// Returns a reference to the inner map.
    pub fn inner(&self) -> &HashMap<u32, String> {
        self.inner.inner()
    }

    /// Returns a mutable reference to the inner map.
    pub fn inner_mut(&mut self) -> &mut HashMap<u32, String> {
        self.inner.inner_mut()
    }
}

#[allow(deprecated)]
impl std::ops::Index<u32> for SymbolMap {
    type Output = String;

    fn index(&self, instrument_id: u32) -> &Self::Output {
        self.get(instrument_id)
            .expect("symbol mapping for instrument ID")
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LiveClient")
            .field("key", &self.key)
            .field("dataset", &self.dataset)
            .field("send_ts_out", &self.send_ts_out)
            .field("upgrade_policy", &self.upgrade_policy)
            .field("session_id", &self.session_id)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
#[allow(deprecated)]
mod tests {
    use std::{ffi::c_char, fmt, time::Duration};

    use dbn::{
        encode::AsyncDbnMetadataEncoder,
        enums::rtype,
        publishers::Dataset,
        record::{HasRType, OhlcvMsg, RecordHeader, TradeMsg, WithTsOut},
        Mbp10Msg, MetadataBuilder, Record, UNDEF_TIMESTAMP,
    };
    use tokio::{join, net::TcpListener, select, sync::mpsc::UnboundedSender, task::JoinHandle};

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
            let stream = self.listener.accept().await.unwrap().0;
            stream.set_nodelay(true).unwrap();
            self.stream = Some(BufReader::new(stream));
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
            // test for partial read bugs
            let half = bytes.len() / 2;
            self.stream().write_all(&bytes[..half]).await.unwrap();
            self.stream().flush().await.unwrap();
            self.stream().write_all(&bytes[half..]).await.unwrap();
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
        Accept,
        Authenticate,
        Send(String),
        Subscribe(Subscription),
        Start,
        SendRecord(Box<dyn AsRef<[u8]> + Send>),
    }

    impl fmt::Debug for Event {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Event::Stop => write!(f, "Stop"),
                Event::Accept => write!(f, "Accept"),
                Event::Authenticate => write!(f, "Authenticate"),
                Event::Send(msg) => write!(f, "Send({msg:?})"),
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
                        Some(Event::Accept) => mock.accept().await,
                        Some(Event::Send(msg)) => mock.send(&msg).await,
                        Some(Event::Subscribe(sub)) => mock.subscribe(sub).await,
                        Some(Event::Start) => mock.start().await,
                        Some(Event::SendRecord(rec)) => mock.send_record(rec).await,
                        Some(Event::Stop) | None => break,
                    }
                }
            });
            Self { task, port, send }
        }

        /// Accept but don't authenticate
        pub fn accept(&mut self) {
            self.send.send(Event::Accept).unwrap();
        }

        /// Accept and authenticate
        pub fn authenticate(&mut self) {
            self.send.send(Event::Authenticate).unwrap();
        }

        pub fn expect_subscribe(&mut self, subscription: Subscription) {
            self.send.send(Event::Subscribe(subscription)).unwrap();
        }

        pub fn start(&mut self) {
            self.send.send(Event::Start).unwrap();
        }

        pub fn send(&mut self, msg: String) {
            self.send.send(Event::Send(msg)).unwrap();
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
        let mut fixture = Fixture::new(dataset.to_string(), send_ts_out).await;
        fixture.authenticate();
        let target = Client::connect_with_addr(
            format!("127.0.0.1:{}", fixture.port),
            "32-character-with-lots-of-filler".to_owned(),
            dataset.to_string(),
            send_ts_out,
            VersionUpgradePolicy::AsIs,
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
        assert_eq!(metadata.version, dbn::DBN_VERSION);
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
        assert_eq!(metadata.version, dbn::DBN_VERSION);
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
            SType::RawSymbol,
            "",
            SType::RawSymbol,
            "AAPL",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        fixture.send_record(SymbolMappingMsg::new(
            2,
            2,
            SType::RawSymbol,
            "",
            SType::RawSymbol,
            "TSLA",
            UNDEF_TIMESTAMP,
            UNDEF_TIMESTAMP,
        )?);
        fixture.send_record(SymbolMappingMsg::new(
            3,
            2,
            SType::RawSymbol,
            "",
            SType::RawSymbol,
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
            SType::RawSymbol,
            "",
            SType::RawSymbol,
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
            SType::RawSymbol,
            "",
            SType::RawSymbol,
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

    #[tokio::test]
    async fn test_error_without_success() {
        const DATASET: Dataset = Dataset::OpraPillar;
        let mut fixture = Fixture::new(DATASET.to_string(), false).await;
        let client_task = tokio::spawn(async move {
            let res = Client::connect_with_addr(
                format!("127.0.0.1:{}", fixture.port),
                "32-character-with-lots-of-filler".to_owned(),
                DATASET.to_string(),
                false,
                VersionUpgradePolicy::AsIs,
            )
            .await;
            if let Err(e) = &res {
                dbg!(e);
            }
            assert!(matches!(res, Err(e) if e.to_string().contains("Unknown failure")));
        });
        let fixture_task = tokio::spawn(async move {
            fixture.accept();

            fixture.send("lsg-test\n".to_owned());
            fixture.send("cram=t7kNhwj4xqR0QYjzFKtBEG2ec2pXJ4FK\n".to_owned());
            fixture.send("Unknown failure\n".to_owned());
        });
        let (r1, r2) = join!(client_task, fixture_task);
        r1.unwrap();
        r2.unwrap();
    }

    #[tokio::test]
    async fn test_cancellation_safety() {
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, true).await;
        fixture.start();
        let metadata = client.start().await.unwrap();
        assert_eq!(metadata.version, dbn::DBN_VERSION);
        assert!(metadata.schema.is_none());
        assert_eq!(metadata.dataset, Dataset::GlbxMdp3.as_str());
        fixture.send_record(Mbp10Msg::default());

        let mut int_1 = tokio::time::interval(Duration::from_millis(1));
        let mut int_2 = tokio::time::interval(Duration::from_millis(1));
        let mut int_3 = tokio::time::interval(Duration::from_millis(1));
        let mut int_4 = tokio::time::interval(Duration::from_millis(1));
        let mut int_5 = tokio::time::interval(Duration::from_millis(1));
        let mut int_6 = tokio::time::interval(Duration::from_millis(1));
        for _ in 0..5_000 {
            select! {
                _ =  int_1.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                _ =  int_2.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                _ =  int_3.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                _ =  int_4.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                _ =  int_5.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                _ =  int_6.tick() => {
                    fixture.send_record(Mbp10Msg::default());
                }
                res = client.next_record() => {
                    let rec = res.unwrap().unwrap();
                    dbg!(rec.header());
                    assert_eq!(*rec.get::<Mbp10Msg>().unwrap(), Mbp10Msg::default());
                }
            }
        }
        fixture.stop().await;
    }
}
