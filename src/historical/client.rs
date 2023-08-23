use reqwest::{header::ACCEPT, IntoUrl, RequestBuilder, Url};

use crate::Error;

use super::{
    batch::BatchClient, metadata::MetadataClient, symbology::SymbologyClient,
    timeseries::TimeseriesClient, HistoricalGateway, API_VERSION,
};

/// The Historical client. Used for symbology resolutions, metadata requests, Historical
/// data older than 24 hours, and submitting batch downloads.
///
/// Use [`HistoricalClient::builder()`](Client::builder) to get a type-safe builder for
/// initializing the required parameters for the client.
///
/// individual API methods are accessed through its four subclients:
/// - [`metadata()`](Self::metadata)
/// - [`timeseries()`](Self::timeseries)
/// - [`symbology()`](Self::symbology)
/// - [`batch()`](Self::batch)
pub struct Client {
    key: String,
    base_url: Url,
    gateway: HistoricalGateway,
    client: reqwest::Client,
}

const USER_AGENT: &str = concat!("Databento/", env!("CARGO_PKG_VERSION"), " Rust");

impl Client {
    /// Returns a type-safe builder for setting the required parameters
    /// for initializing a [`HistoricalClient`](Client).
    pub fn builder() -> ClientBuilder<Unset> {
        ClientBuilder::default()
    }

    /// Creates a new client with the given API key.
    ///
    /// # Errors
    /// This function returns an error when it fails to build the HTTP client.
    pub fn new(key: String, gateway: HistoricalGateway) -> crate::Result<Self> {
        let url = match gateway {
            HistoricalGateway::Bo1 => "https://hist.databento.com",
        };
        Self::with_url(url, key, gateway)
    }

    /// Creates a new client with a specific API URL. This is an advanced method and
    /// [`builder()`](Self::builder) or [`new()`](Self::new) should be used instead.
    ///
    /// # Errors
    /// This function returns an error when the `url` is invalid.
    pub fn with_url(
        url: impl IntoUrl,
        key: String,
        gateway: HistoricalGateway,
    ) -> crate::Result<Self> {
        let base_url = url
            .into_url()
            .map_err(|e| Error::bad_arg("url", format!("{e:?}")))?;
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        Ok(Self {
            key,
            base_url,
            gateway,
            client: reqwest::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .default_headers(headers)
                .build()?,
        })
    }

    /// Returns the API key used by the instance of the client.
    pub fn key(&self) -> &str {
        &self.key
    }

    /// Returns the configured Historical gateway.
    pub fn gateway(&self) -> HistoricalGateway {
        self.gateway
    }

    /// Returns the batch subclient.
    pub fn batch(&mut self) -> BatchClient {
        BatchClient { inner: self }
    }

    /// Returns the metadata subclient.
    pub fn metadata(&mut self) -> MetadataClient {
        MetadataClient { inner: self }
    }

    /// Returns the symbology subclient.
    pub fn symbology(&mut self) -> SymbologyClient {
        SymbologyClient { inner: self }
    }

    /// Returns the timeseries subclient.
    pub fn timeseries(&mut self) -> TimeseriesClient {
        TimeseriesClient { inner: self }
    }

    pub(crate) fn get(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.request(reqwest::Method::GET, slug)
    }

    pub(crate) fn get_with_path(&mut self, path: &str) -> crate::Result<RequestBuilder> {
        Ok(self
            .client
            .get(
                self.base_url
                    .join(path)
                    .map_err(|e| Error::Internal(format!("created invalid URL: {e:?}")))?,
            )
            .basic_auth(&self.key, Option::<&str>::None))
    }

    pub(crate) fn post(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.request(reqwest::Method::POST, slug)
    }

    fn request(&mut self, method: reqwest::Method, slug: &str) -> crate::Result<RequestBuilder> {
        Ok(self
            .client
            .request(
                method,
                self.base_url
                    .join(&format!("v{API_VERSION}/{slug}"))
                    .map_err(|e| Error::Internal(format!("created invalid URL: {e:?}")))?,
            )
            .basic_auth(&self.key, Option::<&str>::None))
    }
}

#[doc(hidden)]
pub struct Unset;

/// A type-safe builder for the [`HistoricalClient`](Client). It will not allow you to
/// call [`Self::build()`] before setting the required `key` field.
pub struct ClientBuilder<AK> {
    key: AK,
    base_url: Option<Url>,
    gateway: HistoricalGateway,
}

impl Default for ClientBuilder<Unset> {
    fn default() -> Self {
        Self {
            key: Unset,
            base_url: None,
            gateway: HistoricalGateway::default(),
        }
    }
}

impl<AK> ClientBuilder<AK> {
    /// Overrides the base URL to be used for the Historical API. Normally this is
    /// derived from the gateway.
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Sets the historical gateway to use.
    pub fn gateway(mut self, gateway: HistoricalGateway) -> Self {
        self.gateway = gateway;
        self
    }
}

impl ClientBuilder<Unset> {
    /// Creates a new [`ClientBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the API key.
    ///
    /// # Errors
    /// This function returns an error when the API key is invalid.
    pub fn key(self, key: impl ToString) -> crate::Result<ClientBuilder<String>> {
        Ok(ClientBuilder {
            key: crate::validate_key(key.to_string())?,
            base_url: self.base_url,
            gateway: self.gateway,
        })
    }

    /// Sets the API key reading it from the `DATABENTO_API_KEY` environment
    /// variable.
    ///
    /// # Errors
    /// This function returns an error when the environment variable is not set or the
    /// API key is invalid.
    pub fn key_from_env(self) -> crate::Result<ClientBuilder<String>> {
        let key = crate::key_from_env()?;
        self.key(key)
    }
}

impl ClientBuilder<String> {
    /// Initializes the client.
    ///
    /// # Errors
    /// This function returns an error when it fails to build the HTTP client.
    pub fn build(self) -> crate::Result<Client> {
        if let Some(url) = self.base_url {
            Client::with_url(url, self.key, self.gateway)
        } else {
            Client::new(self.key, self.gateway)
        }
    }
}
