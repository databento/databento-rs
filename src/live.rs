//! The Live client and related API types. Used for both real-time data and intraday historical.

mod client;

use dbn::{SType, Schema, VersionUpgradePolicy};
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::{ApiKey, Symbols};
pub use client::Client;

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
    #[doc(hidden)]
    /// Reserved for future use.
    #[builder(setter(strip_bool))]
    pub use_snapshot: bool,
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
