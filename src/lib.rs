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

use std::fmt::Write;

#[derive(Debug, Clone)]
pub enum Symbols<'a> {
    AllSymbols,
    Id(u32),
    Ids(&'a [u32]),
    Symbol(&'a str),
    Symbols(&'a [&'a str]),
}

impl<'a> ToString for Symbols<'a> {
    fn to_string(&self) -> String {
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
            Symbols::Symbol(symbol) => symbol.to_owned().to_owned(),
            Symbols::Symbols(symbols) => symbols.join(","),
        }
    }
}
