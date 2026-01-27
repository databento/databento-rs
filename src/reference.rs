//! Reference data client and related API types.

#[doc(inline)]
pub use enums::*;

pub mod adjustment;
pub mod corporate;
mod enums;
pub mod security;

use reqwest::{header::ACCEPT, RequestBuilder, Url};
use time::OffsetDateTime;

use crate::{
    historical::{AddToForm, HistoricalGateway, ReqwestForm, API_VERSION},
    reference::{
        adjustment::AdjustmentFactorsClient, corporate::CorporateActionsClient,
        security::SecurityMasterClient,
    },
    ApiKey, Error, USER_AGENT,
};

/// The reference data client. Used to retrieve security data, corporate actions, and
/// adjustment factors.
///
/// Use [`ReferenceClient::builder()`](Client::builder) to get a type-safe builder for
/// initializing the required parameters for the client.
///
/// Individual API methods are accessed through its three subclients:
/// - [`adjustment_factors()`](Self::adjustment_factors)
/// - [`corporate_actions()`](Self::corporate_actions)
/// - [`security_master()`](Self::security_master)
#[derive(Debug, Clone)]
pub struct Client {
    key: ApiKey,
    base_url: Url,
    gateway: HistoricalGateway,
    client: reqwest::Client,
}

impl Client {
    /// Returns a type-safe builder for setting the required parameters
    /// for initializing a [`ReferenceClient`](Client).
    pub fn builder() -> ClientBuilder<Unset> {
        ClientBuilder::default()
    }

    /// Returns the API key used by the instance of the client.
    pub fn key(&self) -> &str {
        &self.key.0
    }

    /// Returns the configured gateway.
    pub fn gateway(&self) -> HistoricalGateway {
        self.gateway
    }

    /// Returns the adjustment factors subclient.
    pub fn adjustment_factors(&mut self) -> AdjustmentFactorsClient<'_> {
        AdjustmentFactorsClient { inner: self }
    }

    /// Returns the corporate actions subclient.
    pub fn corporate_actions(&mut self) -> CorporateActionsClient<'_> {
        CorporateActionsClient { inner: self }
    }

    /// Returns the security master subclient.
    pub fn security_master(&mut self) -> SecurityMasterClient<'_> {
        SecurityMasterClient { inner: self }
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

#[doc(hidden)]
#[derive(Debug, Copy, Clone)]
pub struct Unset;

/// A type-safe builder for the [`ReferenceClient`](Client). It will not allow you to
/// call [`Self::build()`] before setting the required `key` field.
pub struct ClientBuilder<AK> {
    key: AK,
    base_url: Option<Url>,
    gateway: HistoricalGateway,
    user_agent_ext: Option<String>,
    http_client_builder: Option<reqwest::ClientBuilder>,
}

impl Default for ClientBuilder<Unset> {
    fn default() -> Self {
        Self {
            key: Unset,
            base_url: None,
            gateway: HistoricalGateway::default(),
            user_agent_ext: None,
            http_client_builder: None,
        }
    }
}

impl<AK> ClientBuilder<AK> {
    /// Overrides the base URL to be used for the Reference API. Normally this is
    /// derived from the gateway.
    pub fn base_url(mut self, url: Url) -> Self {
        self.base_url = Some(url);
        self
    }

    /// Sets the gateway to use. Defaults to the BO1 gateway.
    pub fn gateway(mut self, gateway: HistoricalGateway) -> Self {
        self.gateway = gateway;
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
    /// let client = databento::ReferenceClient::builder()
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
            client: http_client,
        })
    }
}

struct Start(OffsetDateTime);
impl AddToForm<Start> for ReqwestForm {
    fn add_to_form(mut self, Start(start): &Start) -> Self {
        self.push(("start", start.unix_timestamp_nanos().to_string()));
        self
    }
}

struct End(Option<OffsetDateTime>);
impl AddToForm<End> for ReqwestForm {
    fn add_to_form(mut self, End(end): &End) -> Self {
        if let Some(end) = end {
            self.push(("end", end.unix_timestamp_nanos().to_string()));
        }
        self
    }
}

impl AddToForm<Vec<Event>> for ReqwestForm {
    fn add_to_form(mut self, events: &Vec<Event>) -> Self {
        if !events.is_empty() {
            self.push((
                "events",
                events
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        self
    }
}

impl AddToForm<Vec<Country>> for ReqwestForm {
    fn add_to_form(mut self, countries: &Vec<Country>) -> Self {
        if !countries.is_empty() {
            self.push((
                "countries",
                countries
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        self
    }
}

impl AddToForm<Vec<SecurityType>> for ReqwestForm {
    fn add_to_form(mut self, security_types: &Vec<SecurityType>) -> Self {
        if !security_types.is_empty() {
            self.push((
                "security_types",
                security_types
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(","),
            ));
        }
        self
    }
}

#[cfg(test)]
mod test_infra {
    use wiremock::MockServer;

    use crate::{historical::test_infra::API_KEY, ReferenceClient};

    pub fn client(mock_server: &MockServer) -> ReferenceClient {
        ReferenceClient::builder()
            .base_url(mock_server.uri().parse().unwrap())
            .key(API_KEY)
            .unwrap()
            .build()
            .unwrap()
    }
}
