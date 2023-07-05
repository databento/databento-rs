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

use crate::{Error, Symbols};

#[derive(Debug, Clone)]
pub struct Subscription<'a> {
    pub symbols: Symbols<'a>,
    pub schema: Schema,
    pub start: Option<OffsetDateTime>,
    pub stype_in: SType,
}

pub struct Client {
    pub api_key: String,
    pub dataset: String,
    pub send_ts_out: bool,
    pub connection: WriteHalf<TcpStream>,
    pub decoder: AsyncRecordDecoder<BufReader<ReadHalf<TcpStream>>>,
    pub session_id: String,
}

const API_KEY_LENGTH: usize = 32;
const BUCKET_ID_LENGTH: usize = 5;

impl Client {
    /// Creates a new client connected to a Live gateway.
    pub async fn connect<A: ToSocketAddrs>(
        api_key: String,
        dataset: String,
        send_ts_out: bool,
    ) -> crate::Result<Self> {
        if api_key.len() != API_KEY_LENGTH {
            return Err(Error::bad_arg(
                "api_key",
                format!("must be of length {API_KEY_LENGTH}"),
            ));
        }
        if api_key.is_ascii() {
            return Err(Error::bad_arg("api_key", "contains non-ASCII characters"));
        }
        Self::connect_with_addr(
            Self::determine_gateway(&dataset),
            api_key,
            dataset,
            send_ts_out,
        )
        .await
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

    /// Creates a new client connected to the Live gateway at `addr`.
    pub async fn connect_with_addr(
        addr: impl ToSocketAddrs,
        api_key: String,
        dataset: String,
        send_ts_out: bool,
    ) -> crate::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        // Authenticate CRAM
        let session_id =
            Self::cram_challenge(&mut reader, &mut writer, &api_key, &dataset, send_ts_out).await?;

        Ok(Self {
            api_key,
            dataset,
            send_ts_out,
            connection: writer,
            decoder: AsyncRecordDecoder::new(reader),
            session_id,
        })
    }

    async fn cram_challenge(
        reader: &mut BufReader<ReadHalf<TcpStream>>,
        writer: &mut WriteHalf<TcpStream>,
        api_key: &str,
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
        let challenge_key = format!("{challenge}|{api_key}");
        let mut hasher = Sha256::new();
        hasher.update(challenge_key.as_bytes());
        let result = hasher.finalize();
        // Safe to slice because Databento only uses ASCII characters in API keys
        let bucket_id = &api_key[API_KEY_LENGTH - BUCKET_ID_LENGTH..];
        let encoded_response = result.encode_hex::<String>();
        let send_ts_out = send_ts_out as i32;
        let reply =
                format!("auth={encoded_response}-{bucket_id}|dataset={dataset}|encoding=dbn|ts_out={send_ts_out}\n");

        // Send CRAM reply
        debug!("[{dataset}] Sending CRAM reply: {reply}");
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

    pub async fn close(mut self) {
        self.connection
            .shutdown()
            .await
            .expect("Error on disconnect");
    }

    pub async fn subscribe(&mut self, sub: &Subscription<'_>) -> crate::Result<()> {
        let Subscription {
            schema, stype_in, ..
        } = &sub;
        let sym_str = sub.symbols.to_string();
        let args = format!("schema={schema}|stype_in={stype_in}|symbols={sym_str}");

        let sub_str = if let Some(start) = sub.start.as_ref() {
            format!("{args}|start={}", start.unix_timestamp_nanos())
        } else {
            args
        };

        debug!("[{}] Subscribing: {sub_str}", self.dataset);
        Ok(self.connection.write_all(sub_str.as_bytes()).await?)
    }

    pub async fn start(&mut self) -> crate::Result<Metadata> {
        info!("[{}] Starting session", self.dataset);
        self.connection.write_all(b"start_session\n").await?;
        Ok(AsyncMetadataDecoder::new(self.decoder.get_mut())
            .decode()
            .await?)
    }

    pub async fn next_record(&mut self) -> crate::Result<Option<RecordRef>> {
        Ok(self.decoder.decode_ref().await?)
    }
}
