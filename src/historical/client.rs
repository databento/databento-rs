use std::io;

use async_compression::tokio::bufread::ZstdDecoder;
use dbn::VersionUpgradePolicy;
use futures::TryStreamExt;
use reqwest::{header::ACCEPT, IntoUrl, RequestBuilder, Url};
use serde::Deserialize;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::{error, warn};

use crate::{error::ApiError, ApiKey, Error, USER_AGENT};

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
/// Individual API methods are accessed through its four subclients:
/// - [`metadata()`](Self::metadata)
/// - [`timeseries()`](Self::timeseries)
/// - [`symbology()`](Self::symbology)
/// - [`batch()`](Self::batch)
#[derive(Debug, Clone)]
pub struct Client {
    key: ApiKey,
    base_url: Url,
    gateway: HistoricalGateway,
    upgrade_policy: VersionUpgradePolicy,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ApiErrorResponse {
    Simple { detail: String },
    Business { detail: BusinessErrorDetails },
}

#[derive(Debug, Deserialize)]
pub(crate) struct BusinessErrorDetails {
    message: String,
    docs: String,
}

pub(crate) const WARNING_HEADER: &str = "X-Warning";
pub(crate) const REQUEST_ID_HEADER: &str = "request-id";

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
        Self::builder().key(key)?.gateway(gateway).build()
    }

    /// Creates a new client with a specific API URL. This is an advanced method and
    /// [`builder()`](Self::builder) or [`new()`](Self::new) should be used instead.
    ///
    /// # Errors
    /// This function returns an error when the `url` is invalid.
    #[deprecated(since = "0.28.0", note = "Use the builder instead")]
    pub fn with_url(
        url: impl IntoUrl,
        key: String,
        gateway: HistoricalGateway,
    ) -> crate::Result<Self> {
        Self::builder()
            .key(key)?
            .gateway(gateway)
            .base_url(url.into_url()?)
            .build()
    }

    /// Returns the API key used by the instance of the client.
    pub fn key(&self) -> &str {
        &self.key.0
    }

    /// Returns the configured Historical gateway.
    pub fn gateway(&self) -> HistoricalGateway {
        self.gateway
    }

    /// Returns the batch subclient.
    pub fn batch(&mut self) -> BatchClient<'_> {
        BatchClient { inner: self }
    }

    /// Returns the metadata subclient.
    pub fn metadata(&mut self) -> MetadataClient<'_> {
        MetadataClient { inner: self }
    }

    /// Returns the symbology subclient.
    pub fn symbology(&mut self) -> SymbologyClient<'_> {
        SymbologyClient { inner: self }
    }

    /// Returns the timeseries subclient.
    pub fn timeseries(&mut self) -> TimeseriesClient<'_> {
        TimeseriesClient { inner: self }
    }

    pub(crate) fn upgrade_policy(&self) -> VersionUpgradePolicy {
        self.upgrade_policy
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
            .basic_auth(self.key(), Option::<&str>::None))
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
            .basic_auth(self.key(), Option::<&str>::None))
    }
}

pub(crate) async fn check_http_error(
    response: reqwest::Response,
) -> crate::Result<reqwest::Response> {
    if response.status().is_success() {
        Ok(response)
    } else {
        let request_id = response
            .headers()
            .get(REQUEST_ID_HEADER)
            .and_then(|header| header.to_str().ok().map(ToOwned::to_owned));
        let status_code = response.status();
        let body = response.text().await.unwrap_or_default();
        let err = match serde_json::from_str::<ApiErrorResponse>(&body) {
            Ok(ApiErrorResponse::Simple { detail: message }) => ApiError {
                request_id,
                status_code,
                message,
                docs_url: None,
            },
            Ok(ApiErrorResponse::Business { detail }) => ApiError {
                request_id,
                status_code,
                message: detail.message,
                docs_url: Some(detail.docs),
            },
            Err(e) => {
                warn!("Failed to deserialize error response to expected JSON format: {e:?}");
                ApiError {
                    request_id,
                    status_code,
                    message: body,
                    docs_url: None,
                }
            }
        };
        Err(Error::Api(err))
    }
}

pub(crate) async fn handle_response<R: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> crate::Result<R> {
    check_warnings(&response);
    let response = check_http_error(response).await?;
    let bytes = response.bytes().await?;

    deserialize_json(std::str::from_utf8(&bytes)?)
}

pub(crate) async fn handle_zstd_jsonl_response<R: serde::de::DeserializeOwned>(
    response: reqwest::Response,
) -> crate::Result<Vec<R>> {
    check_warnings(&response);
    let response = check_http_error(response).await?;
    let stream = response.bytes_stream().map_err(io::Error::other);
    let stream_reader = tokio_util::io::StreamReader::new(stream);
    let mut lines_decoder = BufReader::new(ZstdDecoder::new(stream_reader)).lines();
    let mut res = Vec::new();
    while let Some(line) = lines_decoder
        .next_line()
        .await
        .inspect_err(|err| error!(?err, "Failed decoding zstd-compressed JSON line"))?
    {
        res.push(deserialize_json(&line)?);
    }
    Ok(res)
}

fn deserialize_json<R: serde::de::DeserializeOwned>(str: &str) -> crate::Result<R> {
    serde_json::from_str(str).map_err(|err| {
        error!(?err, ?str, "Failed to decode JSON");
        crate::Error::from(err)
    })
}

pub(crate) fn check_warnings(response: &reqwest::Response) {
    if let Some(header) = response.headers().get(WARNING_HEADER) {
        match serde_json::from_slice::<Vec<String>>(header.as_bytes()) {
            Ok(warnings) => {
                for warning in warnings {
                    warn!("{warning}");
                }
            }
            Err(err) => {
                warn!(?err, "Failed to parse server warnings from HTTP header");
            }
        };
    };
}

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct Unset;

/// A type-safe builder for the [`HistoricalClient`](Client). It will not allow you to
/// call [`Self::build()`] before setting the required `key` field.
pub struct ClientBuilder<AK> {
    key: AK,
    base_url: Option<Url>,
    gateway: HistoricalGateway,
    upgrade_policy: VersionUpgradePolicy,
    user_agent_ext: Option<String>,
    http_client_builder: Option<reqwest::ClientBuilder>,
}

impl Default for ClientBuilder<Unset> {
    fn default() -> Self {
        Self {
            key: Unset,
            base_url: None,
            gateway: HistoricalGateway::default(),
            upgrade_policy: VersionUpgradePolicy::default(),
            user_agent_ext: None,
            http_client_builder: None,
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

    /// Sets the historical gateway to use. Defaults to the BO1 gateway.
    pub fn gateway(mut self, gateway: HistoricalGateway) -> Self {
        self.gateway = gateway;
        self
    }

    /// Sets the DBN version upgrade policy to be used in the
    /// [`Client::timeseries()`] methods.
    pub fn upgrade_policy(mut self, upgrade_policy: VersionUpgradePolicy) -> Self {
        self.upgrade_policy = upgrade_policy;
        self
    }

    /// Extends the user agent. Intended for library authors.
    pub fn user_agent_extension(mut self, extension: String) -> Self {
        self.user_agent_ext = Some(extension);
        self
    }

    /// Provides a custom [`reqwest::ClientBuilder`] for advanced HTTP client
    /// configuration.
    ///
    /// This allows configuring timeouts, TLS settings, and other HTTP client options.
    /// The builder will be finalized with Databento's settings (user-agent and default
    /// headers).
    ///
    /// # Example
    /// ```
    /// use std::time::Duration;
    ///
    /// let client = databento::HistoricalClient::builder()
    ///     .key("db-xxxxxxxxxxxxxxxxxxxxxxxxxxxxx")?
    ///     .http_client_builder(
    ///         reqwest::ClientBuilder::new()
    ///             .timeout(Duration::from_secs(60))
    ///             .connect_timeout(Duration::from_secs(10))
    ///     )
    ///     .build()?;
    /// # Ok::<(), databento::Error>(())
    /// ```
    pub fn http_client_builder(mut self, builder: reqwest::ClientBuilder) -> Self {
        self.http_client_builder = Some(builder);
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
    pub fn key(self, key: impl ToString) -> crate::Result<ClientBuilder<ApiKey>> {
        Ok(ClientBuilder {
            key: ApiKey::new(key.to_string())?,
            base_url: self.base_url,
            gateway: self.gateway,
            upgrade_policy: self.upgrade_policy,
            user_agent_ext: self.user_agent_ext,
            http_client_builder: self.http_client_builder,
        })
    }

    /// Sets the API key reading it from the `DATABENTO_API_KEY` environment
    /// variable.
    ///
    /// # Errors
    /// This function returns an error when the environment variable is not set or the
    /// API key is invalid.
    pub fn key_from_env(self) -> crate::Result<ClientBuilder<ApiKey>> {
        let key = crate::key_from_env()?;
        self.key(key)
    }
}

impl ClientBuilder<ApiKey> {
    /// Initializes the client.
    ///
    /// # Errors
    /// This function returns an error when it fails to build the HTTP client.
    pub fn build(self) -> crate::Result<Client> {
        let base_url = if let Some(url) = self.base_url {
            url
        } else {
            self.gateway
                .as_url()
                .parse()
                .map_err(|e| Error::bad_arg("gateway", format!("{e:?}")))?
        };
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(ACCEPT, "application/json".parse().unwrap());
        let user_agent = self
            .user_agent_ext
            .map(|ext| format!("{} {ext}", *USER_AGENT))
            .unwrap_or_else(|| USER_AGENT.clone());
        let http_client = self
            .http_client_builder
            .unwrap_or_default()
            .user_agent(user_agent)
            .default_headers(headers)
            .build()?;
        Ok(Client {
            key: self.key,
            base_url,
            gateway: self.gateway,
            upgrade_policy: self.upgrade_policy,
            client: http_client,
        })
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use wiremock::{matchers::method, Mock, MockServer, ResponseTemplate};

    use super::*;

    #[tokio::test]
    async fn check_http_error_non_json() {
        const BODY: &str = "<html><body><h1>502 Bad Gateway</h1>
The server returned an invalid or incomplete response.
</body></html>";
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .respond_with(
                ResponseTemplate::new(StatusCode::BAD_GATEWAY.as_u16()).set_body_string(BODY),
            )
            .mount(&mock_server)
            .await;
        let resp = reqwest::get(mock_server.uri()).await.unwrap();
        let err = check_http_error(resp).await.unwrap_err();
        assert!(
            matches!(err, Error::Api(api_err) if api_err.status_code == StatusCode::BAD_GATEWAY && api_err.message == BODY && api_err.docs_url.is_none())
        );
    }
}
