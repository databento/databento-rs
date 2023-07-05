use reqwest::header::{ACCEPT, WWW_AUTHENTICATE};

use super::{
    batch::BatchClient, metadata::MetadataClient, symbology::SymbologyClient,
    timeseries::TimeseriesClient, HistoricalGateway,
};

pub struct Client {
    api_key: String,
    gateway: HistoricalGateway,
    client: reqwest::Client,
}

const USER_AGENT: &str = concat!("Databento/", env!("CARGO_PKG_VERSION"), " Rust");

impl Client {
    pub fn new(api_key: String, gateway: HistoricalGateway) -> crate::Result<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        headers.insert(
            WWW_AUTHENTICATE,
            format!(
                "Basic {}:",
                base64::display::Base64Display::new(
                    api_key.as_bytes(),
                    &base64::engine::general_purpose::STANDARD
                )
            )
            .parse()
            .unwrap(),
        );
        Ok(Self {
            api_key,
            gateway,
            client: reqwest::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .default_headers(headers)
                .build()?,
        })
    }

    pub fn key(&self) -> &str {
        &self.api_key
    }

    pub fn gateway(&self) -> HistoricalGateway {
        self.gateway
    }

    pub fn batch(&mut self) -> BatchClient {
        BatchClient {
            inner: &mut self.client,
        }
    }

    pub fn metadata(&mut self) -> MetadataClient {
        MetadataClient {
            inner: &mut self.client,
        }
    }

    pub fn symbology(&mut self) -> SymbologyClient {
        SymbologyClient {
            inner: &mut self.client,
        }
    }

    pub fn timeseries(&mut self) -> TimeseriesClient {
        TimeseriesClient {
            inner: &mut self.client,
        }
    }
}
