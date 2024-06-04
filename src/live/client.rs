use std::{collections::HashMap, fmt};

use dbn::{
    decode::dbn::{AsyncMetadataDecoder, AsyncRecordDecoder},
    Metadata, RecordRef, VersionUpgradePolicy,
};
use hex::ToHex;
use log::{debug, error, info};
use sha2::{Digest, Sha256};
use time::Duration;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{validate_key, ApiKey, Error, API_KEY_LENGTH, BUCKET_ID_LENGTH};

use super::{ClientBuilder, Subscription, Unset};

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
    /// This function returns an error when `key` is invalid or it's unable to connect
    /// and authenticate with the Live gateway.
    /// This function returns an error when `key` or `heartbeat_interval` are invalid,
    /// or it's unable to connect and authenticate with the Live gateway.
    pub async fn connect(
        key: String,
        dataset: String,
        send_ts_out: bool,
        upgrade_policy: VersionUpgradePolicy,
        heartbeat_interval: Option<Duration>,
    ) -> crate::Result<Self> {
        Self::connect_with_addr(
            Self::determine_gateway(&dataset),
            key,
            dataset,
            send_ts_out,
            upgrade_policy,
            heartbeat_interval,
        )
        .await
    }

    /// Creates a new client connected to the Live gateway at `addr`. This is an advanced method and generally
    /// [`builder()`](Self::builder) or [`connect()`](Self::connect) should be used instead.
    ///
    /// # Errors
    /// This function returns an error when `key` or `heartbeat_interval` are invalid,
    /// or it's unable to connect and authenticate with the Live gateway.
    pub async fn connect_with_addr(
        addr: impl ToSocketAddrs,
        key: String,
        dataset: String,
        send_ts_out: bool,
        upgrade_policy: VersionUpgradePolicy,
        heartbeat_interval: Option<Duration>,
    ) -> crate::Result<Self> {
        let key = validate_key(key)?;
        let stream = TcpStream::connect(addr).await?;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        // Authenticate CRAM
        let session_id = Self::cram_challenge(
            &mut reader,
            &mut writer,
            &key.0,
            &dataset,
            send_ts_out,
            heartbeat_interval,
        )
        .await?;

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
        heartbeat_interval: Option<Duration>,
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
        let mut reply =
                format!("auth={encoded_response}-{bucket_id}|dataset={dataset}|encoding=dbn|ts_out={send_ts_out}|client=Rust {}", env!("CARGO_PKG_VERSION"));
        if let Some(heartbeat_interval_s) = heartbeat_interval.map(|i| i.whole_seconds()) {
            reply = format!("{reply}|heartbeat_interval_s={heartbeat_interval_s}\n")
        } else {
            reply.push('\n');
        }

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
            schema,
            stype_in,
            start,
            use_snapshot,
            ..
        } = &sub;

        if *use_snapshot && start.is_some() {
            return Err(Error::BadArgument {
                param_name: "use_snapshot".to_string(),
                desc: "cannot request snapshot with start time".to_string(),
            });
        }

        for sym_str in sub.symbols.to_chunked_api_string() {
            let snapshot = *use_snapshot as u8;

            let args = format!(
                "schema={schema}|stype_in={stype_in}|symbols={sym_str}|snapshot={snapshot}"
            );

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
    use std::{ffi::c_char, fmt};

    use dbn::{
        encode::AsyncDbnMetadataEncoder,
        enums::rtype,
        publishers::Dataset,
        record::{HasRType, OhlcvMsg, RecordHeader, TradeMsg, WithTsOut},
        FlagSet, Mbp10Msg, MetadataBuilder, Record, SType, Schema,
    };
    use time::Duration;
    use tokio::{
        io::BufReader,
        join,
        net::{TcpListener, TcpStream},
        select,
        sync::mpsc::UnboundedSender,
        task::JoinHandle,
    };

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

        async fn authenticate(&mut self, heartbeat_interval: Option<Duration>) {
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
            if let Some(heartbeat_interval) = heartbeat_interval {
                assert!(auth_line.contains(&format!(
                    "heartbeat_interval_s={}",
                    heartbeat_interval.whole_seconds()
                )));
            } else {
                assert!(!auth_line.contains("heartbeat_interval_s="));
            }
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

            if subscription.use_snapshot {
                assert!(sub_line.contains("snapshot=1"));
            } else {
                assert!(sub_line.contains("snapshot=0"));
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
        Authenticate(Option<Duration>),
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
                Event::Authenticate(hb_int) => write!(f, "Authenticate({hb_int:?})"),
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
                        Some(Event::Authenticate(hb_int)) => mock.authenticate(hb_int).await,
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
        pub fn authenticate(&mut self, heartbeat_interval: Option<Duration>) {
            self.send
                .send(Event::Authenticate(heartbeat_interval))
                .unwrap();
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

    async fn setup(
        dataset: Dataset,
        send_ts_out: bool,
        heartbeat_interval: Option<Duration>,
    ) -> (Fixture, Client) {
        let _ = env_logger::try_init();
        let mut fixture = Fixture::new(dataset.to_string(), send_ts_out).await;
        fixture.authenticate(heartbeat_interval);
        let builder = Client::builder()
            .addr(format!("127.0.0.1:{}", fixture.port))
            .await
            .unwrap()
            .key("32-character-with-lots-of-filler".to_owned())
            .unwrap()
            .dataset(dataset.to_string())
            .send_ts_out(send_ts_out);
        let target = if let Some(heartbeat_interval) = heartbeat_interval {
            builder.heartbeat_interval(heartbeat_interval)
        } else {
            builder
        }
        .build()
        .await
        .unwrap();
        (fixture, target)
    }

    #[tokio::test]
    async fn test_subscribe() {
        let (mut fixture, mut client) = setup(Dataset::XnasItch, false, None).await;
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
    async fn test_subscribe_snapshot() {
        let (mut fixture, mut client) =
            setup(Dataset::XnasItch, false, Some(Duration::MINUTE)).await;
        let subscription = Subscription::builder()
            .symbols(vec!["MSFT", "TSLA", "QQQ"])
            .schema(Schema::Ohlcv1M)
            .stype_in(SType::RawSymbol)
            .use_snapshot()
            .build();
        fixture.expect_subscribe(subscription.clone());
        client.subscribe(&subscription).await.unwrap();
        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_subscribe_snapshot_failed() {
        let (fixture, mut client) =
            setup(Dataset::XnasItch, false, Some(Duration::seconds(5))).await;

        let err = client
            .subscribe(
                &Subscription::builder()
                    .symbols(vec!["MSFT", "TSLA", "QQQ"])
                    .schema(Schema::Ohlcv1M)
                    .stype_in(SType::RawSymbol)
                    .start(time::OffsetDateTime::now_utc())
                    .use_snapshot()
                    .build(),
            )
            .await
            .unwrap_err();
        assert!(err
            .to_string()
            .contains("cannot request snapshot with start time"));

        fixture.stop().await;
    }

    #[tokio::test]
    async fn test_subscription_chunking() {
        const SYMBOL: &str = "TEST";
        const SYMBOL_COUNT: usize = 1000;
        let (mut fixture, mut client) = setup(Dataset::XnasItch, false, None).await;
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
        let (mut fixture, mut client) =
            setup(Dataset::GlbxMdp3, false, Some(Duration::minutes(5))).await;
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
                flags: FlagSet::default(),
                depth: 1,
                ts_recv: 0,
                ts_in_delta: 0,
                sequence: 2,
            },
            time::OffsetDateTime::now_utc().unix_timestamp_nanos() as u64,
        );
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, true, None).await;
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
    async fn test_close() {
        let (mut fixture, mut client) =
            setup(Dataset::GlbxMdp3, true, Some(Duration::seconds(45))).await;
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
            let res = Client::builder()
                .addr(format!("127.0.0.1:{}", fixture.port))
                .await
                .unwrap()
                .key("32-character-with-lots-of-filler".to_owned())
                .unwrap()
                .dataset(DATASET.to_string())
                .build()
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
        let (mut fixture, mut client) = setup(Dataset::GlbxMdp3, true, None).await;
        fixture.start();
        let metadata = client.start().await.unwrap();
        assert_eq!(metadata.version, dbn::DBN_VERSION);
        assert!(metadata.schema.is_none());
        assert_eq!(metadata.dataset, Dataset::GlbxMdp3.as_str());
        fixture.send_record(Mbp10Msg::default());

        let mut int_1 = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut int_2 = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut int_3 = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut int_4 = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut int_5 = tokio::time::interval(std::time::Duration::from_millis(1));
        let mut int_6 = tokio::time::interval(std::time::Duration::from_millis(1));
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
