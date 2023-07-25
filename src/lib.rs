//! The official Databento client library.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::missing_errors_doc)]

pub mod error;
#[cfg(feature = "historical")]
pub mod historical;
#[cfg(feature = "live")]
pub mod live;

pub use error::{Error, Result};
#[cfg(feature = "historical")]
pub use historical::Client as HistoricalClient;
#[cfg(feature = "live")]
pub use live::Client as LiveClient;
// Re-export to keep versions synchronized
pub use dbn;

use std::fmt::{Display, Write};

/// One or more symbols in a particular [`SType`](dbn::enums::SType).
#[derive(Debug, Clone)]
pub enum Symbols {
    /// Sentinel value for all symbols in a dataset.
    All,
    /// A single symbol identified by its instrument ID.
    Id(u32),
    /// A set of symbols identified by their instrument IDs.
    Ids(Vec<u32>),
    /// A single symbol.
    Symbol(String),
    /// A set of symbols.
    Symbols(Vec<String>),
}

const ALL_SYMBOLS: &str = "ALL_SYMBOLS";

impl Symbols {
    /// Returns the string representation for sending to the API.
    pub fn to_api_string(&self) -> String {
        match self {
            Symbols::All => ALL_SYMBOLS.to_owned(),
            Symbols::Id(id) => id.to_string(),
            Symbols::Ids(ids) => ids.iter().fold(String::new(), |mut acc, s| {
                if acc.is_empty() {
                    s.to_string()
                } else {
                    write!(acc, ",{s}").unwrap();
                    acc
                }
            }),
            Symbols::Symbol(symbol) => symbol.to_owned(),
            Symbols::Symbols(symbols) => symbols.join(","),
        }
    }
}

impl Display for Symbols {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Symbols::All => f.write_str(ALL_SYMBOLS),
            Symbols::Id(id) => write!(f, "{id}"),
            Symbols::Ids(ids) => {
                for (i, id) in ids.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{id}")?;
                    } else {
                        write!(f, ", {id}")?;
                    }
                }
                Ok(())
            }
            Symbols::Symbol(symbol) => f.write_str(symbol),
            Symbols::Symbols(symbols) => {
                for (i, sym) in symbols.iter().enumerate() {
                    if i == 0 {
                        write!(f, "{sym}")?;
                    } else {
                        write!(f, ", {sym}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

impl From<&str> for Symbols {
    fn from(value: &str) -> Self {
        Symbols::Symbol(value.to_owned())
    }
}

impl From<u32> for Symbols {
    fn from(value: u32) -> Self {
        Symbols::Id(value)
    }
}

impl From<Vec<u32>> for Symbols {
    fn from(value: Vec<u32>) -> Self {
        Symbols::Ids(value)
    }
}

impl From<String> for Symbols {
    fn from(value: String) -> Self {
        Symbols::Symbol(value)
    }
}

impl From<Vec<String>> for Symbols {
    fn from(value: Vec<String>) -> Self {
        Symbols::Symbols(value)
    }
}

impl From<Vec<&str>> for Symbols {
    fn from(value: Vec<&str>) -> Self {
        Symbols::Symbols(value.into_iter().map(ToOwned::to_owned).collect())
    }
}

pub(crate) fn validate_key(key: String) -> crate::Result<String> {
    if key.len() != 32 {
        Err(Error::bad_arg("key", "expected to be 32-characters long"))
    } else if !key.is_ascii() {
        Err(Error::bad_arg(
            "key",
            "expected to be composed of only ASCII characters",
        ))
    } else {
        Ok(key)
    }
}

pub(crate) fn key_from_env() -> crate::Result<String> {
    std::env::var("DATABENTO_API_KEY").map_err(|e| Error::bad_arg("key", format!("{e:?}")))
}

#[cfg(test)]
const TEST_DATA_PATH: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data");
#[cfg(test)]
pub(crate) fn zst_test_data_path(schema: dbn::enums::Schema) -> String {
    format!("{TEST_DATA_PATH}/test_data.{}.dbn.zst", schema.as_str())
}
#[cfg(test)]
pub(crate) fn body_contains(
    key: impl Display,
    val: impl Display,
) -> wiremock::matchers::BodyContainsMatcher {
    wiremock::matchers::body_string_contains(&format!("{key}={val}"))
}
