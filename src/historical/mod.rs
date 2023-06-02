pub mod batch;
mod client;
pub mod metadata;
pub mod symbology;
pub mod timeseries;

pub use client::*;

use crate::Symbols;

pub const API_VERSION: u32 = 0;
pub const API_VERSION_STR: &str = "0";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoricalGateway {
    #[default]
    Bo1,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateRange {
    FwdFill(time::Date),
    Closed { start: time::Date, end: time::Date },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateTimeRange {
    FwdFill(time::OffsetDateTime),
    Closed {
        start: time::OffsetDateTime,
        end: time::OffsetDateTime,
    },
}

trait AddToQuery<T> {
    fn add_to_query(self, param: &T) -> Self;
}

impl AddToQuery<DateTimeRange> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &DateTimeRange) -> Self {
        match param {
            DateTimeRange::FwdFill(start) => self.query(&[("start", start)]),
            DateTimeRange::Closed { start, end } => self.query(&[("start", start), ("end", end)]),
        }
    }
}

impl AddToQuery<Symbols<'_>> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &Symbols<'_>) -> Self {
        self.query(&[("symbols", param.to_string())])
    }
}
