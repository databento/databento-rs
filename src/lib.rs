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

use std::fmt::{Display, Write};

/// One or more symbols in a particular [`SType`](dbn::enums::SType).
#[derive(Debug, Clone)]
pub enum Symbols {
    /// Sentinel value for all symbols in a dataset.
    AllSymbols,
    /// A single symbol identified by its instrument ID.
    Id(u32),
    /// A set of symbols identified by their instrument IDs.
    Ids(Vec<u32>),
    /// A single symbol.
    Symbol(String),
    /// A set of symbols.
    Symbols(Vec<String>),
}

impl Symbols {
    /// Returns the string representation for sending to the API.
    pub fn to_api_string(&self) -> String {
        match self {
            Symbols::AllSymbols => "ALL_SYMBOLS".to_owned(),
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
            Symbols::AllSymbols => f.write_str("ALL_SYMBOLS"),
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
