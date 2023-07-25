//! The Live client and related API types. Used for both real-time data and intraday historical.

use std::collections::HashMap;

use dbn::{
    decode::dbn::{AsyncMetadataDecoder, AsyncRecordDecoder},
    enums::{SType, Schema},
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

use crate::{Error, Symbols};

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

const API_KEY_LENGTH: usize = 32;
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
        if key.len() != API_KEY_LENGTH {
            return Err(Error::bad_arg(
                "key",
                format!("must be of length {API_KEY_LENGTH}"),
            ));
        }
        if key.is_ascii() {
            return Err(Error::bad_arg("key", "contains non-ASCII characters"));
        }
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
                format!("auth={encoded_response}-{bucket_id}|dataset={dataset}|encoding=dbn|ts_out={send_ts_out}\n");

        // Send CRAM reply
        debug!(
            "[{dataset}] Sending CRAM reply: {}",
            &reply[..reply.len() - 1]
        );
        writer.write_all(reply.as_bytes()).await.unwrap();

        response.clear();
        reader.read_line(&mut response).await?;

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

    /// Closes the connection with the gateway, ending the session and all subscriptions.
    pub async fn close(mut self) {
        self.connection
            .shutdown()
            .await
            .expect("Error on disconnect");
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
        let sym_str = sub.symbols.to_api_string();
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
        Ok(self.connection.write_all(sub_str.as_bytes()).await?)
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

    /// Fetches the next record
    /// This method should only be called after the session has been [started](Self::start).
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
    pub symbols: Symbols,
    /// The schema of data to subscribe to.
    pub schema: Schema,
    /// If specified, requests available data since that time. When `None`,
    /// only real-time data is sent.
    ///
    /// Setting this field is not supported once the session has been started with
    /// [`LiveClient::start`](crate::LiveClient::start).
    #[builder(default)]
    pub start: Option<OffsetDateTime>,
    /// The symbology type of symbols in [`symbols`](Self::symbols).
    pub stype_in: SType,
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
    pub fn key(self, key: String) -> crate::Result<ClientBuilder<String, D>> {
        Ok(ClientBuilder {
            key: crate::validate_key(key)?,
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
    pub fn dataset(self, dataset: String) -> ClientBuilder<AK, String> {
        ClientBuilder {
            key: self.key,
            dataset,
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
