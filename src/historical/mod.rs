//! Historical client and related API types.

pub mod batch;
mod client;
pub mod metadata;
pub mod symbology;
pub mod timeseries;

pub use client::*;
use time::{format_description::FormatItem, macros::format_description};

use crate::{Error, Symbols};

/// The current Databento historical API version.
pub const API_VERSION: u32 = 0;

/// The Historical API gateway to use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoricalGateway {
    /// The default gateway in Boston.
    #[default]
    Bo1,
}

/// A date range query. It can either be closed, or use
/// forward fill behavior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateRange {
    /// An interval where `end` is unspecified.
    Open(time::Date),
    /// A closed interval with an inclusive start date and an exclusive end date.
    Closed {
        /// The start date (inclusive).
        start: time::Date,
        /// The end date (exclusive).
        end: time::Date,
    },
}

/// A date time range query. It can either be closed, or use
/// forward fill behavior.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateTimeRange {
    /// An interval where `end` is implied.
    FwdFill(time::OffsetDateTime),
    /// A closed interval with an inclusive start time and an exclusive end time.
    Closed {
        /// The start date time (inclusive).
        start: time::OffsetDateTime,
        /// The end date time (inclusive).
        end: time::OffsetDateTime,
    },
}

impl From<(time::Date, time::Date)> for DateRange {
    fn from(value: (time::Date, time::Date)) -> Self {
        Self::Closed {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<time::Date> for DateRange {
    fn from(value: time::Date) -> Self {
        Self::Open(value)
    }
}

pub(crate) const DATE_FORMAT: &[FormatItem<'_>] = format_description!("[year]-[month]-[day]");

impl From<(time::OffsetDateTime, time::OffsetDateTime)> for DateTimeRange {
    fn from(value: (time::OffsetDateTime, time::OffsetDateTime)) -> Self {
        Self::Closed {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<time::OffsetDateTime> for DateTimeRange {
    fn from(value: time::OffsetDateTime) -> Self {
        Self::FwdFill(value)
    }
}

impl TryFrom<(u64, u64)> for DateTimeRange {
    type Error = crate::Error;

    fn try_from(value: (u64, u64)) -> Result<Self, Self::Error> {
        let start = time::OffsetDateTime::from_unix_timestamp_nanos(value.0 as i128)
            .map_err(|e| Error::bad_arg("first UNIX nanos", format!("{e:?}")))?;
        let end = time::OffsetDateTime::from_unix_timestamp_nanos(value.1 as i128)
            .map_err(|e| Error::bad_arg("second UNIX nanos", format!("{e:?}")))?;
        Ok(Self::Closed { start, end })
    }
}

trait AddToQuery<T> {
    fn add_to_query(self, param: &T) -> Self;
}

impl AddToQuery<DateRange> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &DateRange) -> Self {
        match param {
            DateRange::Open(start) => {
                self.query(&[("start_date", start.format(DATE_FORMAT).unwrap())])
            }
            DateRange::Closed { start, end } => self.query(&[
                ("start_date", start.format(DATE_FORMAT).unwrap()),
                ("end_date", end.format(DATE_FORMAT).unwrap()),
            ]),
        }
    }
}

impl AddToQuery<DateTimeRange> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &DateTimeRange) -> Self {
        match param {
            DateTimeRange::FwdFill(start) => self.query(&[("start", start.unix_timestamp_nanos())]),
            DateTimeRange::Closed { start, end } => self.query(&[
                ("start", start.unix_timestamp_nanos()),
                ("end", end.unix_timestamp_nanos()),
            ]),
        }
    }
}

impl AddToQuery<Symbols> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &Symbols) -> Self {
        self.query(&[("symbols", param.to_api_string())])
    }
}

impl DateRange {
    pub(crate) fn add_to_form(&self, form: &mut Vec<(&'static str, String)>) {
        match self {
            DateRange::Open(start) => {
                form.push(("start_date", start.format(DATE_FORMAT).unwrap()));
            }
            DateRange::Closed { start, end } => {
                form.push(("start_date", start.format(DATE_FORMAT).unwrap()));
                form.push(("end_date", end.format(DATE_FORMAT).unwrap()));
            }
        }
    }
}

impl DateTimeRange {
    pub(crate) fn add_to_form(&self, form: &mut Vec<(&'static str, String)>) {
        match self {
            DateTimeRange::FwdFill(start) => {
                form.push(("start", start.unix_timestamp_nanos().to_string()));
            }
            DateTimeRange::Closed { start, end } => {
                form.push(("start", start.unix_timestamp_nanos().to_string()));
                form.push(("end", end.unix_timestamp_nanos().to_string()));
            }
        }
    }
}
