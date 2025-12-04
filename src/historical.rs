//! Historical client and related API types.

pub mod batch;
mod client;
pub mod metadata;
pub mod symbology;
pub mod timeseries;

use std::{
    num::NonZeroU64,
    ops::{Range, RangeInclusive},
};

pub use client::*;
use serde::Deserialize;
use time::{
    format_description::BorrowedFormatItem, macros::format_description, Duration, Time, UtcOffset,
};

use crate::{deserialize::deserialize_date_time, Error, Symbols};

/// The current Databento historical API version.
pub const API_VERSION: u32 = 0;

/// The Historical API gateway to use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HistoricalGateway {
    /// The default gateway in Boston.
    #[default]
    Bo1,
}

impl HistoricalGateway {
    /// Returns the URL string associated with the gateway.
    pub fn as_url(&self) -> &str {
        match self {
            HistoricalGateway::Bo1 => "https://hist.databento.com",
        }
    }
}

/// A **half**-closed date interval with an inclusive UTC start date and an exclusive UTC end
/// date.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DateRange {
    /// The inclusive UTC start date.
    pub start: time::Date,
    /// The exclusive UTC end date.
    pub end: time::Date,
}

/// A **half**-closed datetime interval with an inclusive start and an exclusive end.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct DateTimeRange {
    /// The inclusive start.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub start: time::OffsetDateTime,
    /// The exclusive end.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub end: time::OffsetDateTime,
}

impl From<Range<time::Date>> for DateRange {
    fn from(range: Range<time::Date>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<time::Date>> for DateRange {
    fn from(range: RangeInclusive<time::Date>) -> Self {
        Self {
            start: *range.start(),
            end: range.end().next_day().unwrap(),
        }
    }
}

impl From<(time::Date, time::Date)> for DateRange {
    fn from(value: (time::Date, time::Date)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<(time::Date, time::Duration)> for DateRange {
    fn from(value: (time::Date, time::Duration)) -> Self {
        Self {
            start: value.0,
            end: value.0 + value.1,
        }
    }
}

impl From<Range<time::Date>> for DateTimeRange {
    fn from(range: Range<time::Date>) -> Self {
        Self::from(DateRange::from(range))
    }
}

impl From<RangeInclusive<time::Date>> for DateTimeRange {
    fn from(range: RangeInclusive<time::Date>) -> Self {
        Self::from(DateRange::from(range))
    }
}

impl From<Range<time::OffsetDateTime>> for DateTimeRange {
    fn from(range: Range<time::OffsetDateTime>) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

impl From<RangeInclusive<time::OffsetDateTime>> for DateTimeRange {
    fn from(range: RangeInclusive<time::OffsetDateTime>) -> Self {
        Self {
            start: *range.start(),
            end: *range.end() + time::Duration::NANOSECOND,
        }
    }
}

impl From<(time::Date, time::Duration)> for DateTimeRange {
    fn from(value: (time::Date, time::Duration)) -> Self {
        Self::from(DateRange::from(value))
    }
}

impl From<(time::Date, time::Date)> for DateTimeRange {
    fn from(value: (time::Date, time::Date)) -> Self {
        Self::from(DateRange::from(value))
    }
}

impl From<time::Date> for DateRange {
    fn from(date: time::Date) -> Self {
        Self {
            start: date,
            end: date.next_day().unwrap(),
        }
    }
}

impl From<time::Date> for DateTimeRange {
    fn from(date: time::Date) -> Self {
        let start = date.with_time(Time::MIDNIGHT).assume_utc();
        Self {
            start,
            end: start + Duration::DAY,
        }
    }
}

impl From<DateRange> for DateTimeRange {
    fn from(date_range: DateRange) -> Self {
        Self {
            start: date_range.start.with_time(Time::MIDNIGHT).assume_utc(),
            end: date_range.end.with_time(Time::MIDNIGHT).assume_utc(),
        }
    }
}

impl From<DateTimeRange> for DateRange {
    fn from(dt_range: DateTimeRange) -> Self {
        let utc_end = dt_range.end.to_offset(UtcOffset::UTC);
        Self {
            start: dt_range.start.to_offset(UtcOffset::UTC).date(),
            // Round up end to nearest date
            end: if utc_end.time() == Time::MIDNIGHT {
                utc_end.date()
            } else {
                utc_end.date().next_day().unwrap()
            },
        }
    }
}

pub(crate) const DATE_FORMAT: &[BorrowedFormatItem<'_>] =
    format_description!("[year]-[month]-[day]");

impl From<(time::OffsetDateTime, time::OffsetDateTime)> for DateTimeRange {
    fn from(value: (time::OffsetDateTime, time::OffsetDateTime)) -> Self {
        Self {
            start: value.0,
            end: value.1,
        }
    }
}

impl From<(time::OffsetDateTime, time::Duration)> for DateTimeRange {
    fn from(value: (time::OffsetDateTime, time::Duration)) -> Self {
        Self {
            start: value.0,
            end: value.0 + value.1,
        }
    }
}

#[cfg(feature = "chrono")]
mod chrono_impl {
    use super::{DateRange, DateTimeRange};

    fn chrono_datetime_to_time(dt: chrono::DateTime<chrono::Utc>) -> time::OffsetDateTime {
        // timestamp_nanos_opt() returns None for dates outside ~1677-2262.
        // from_unix_timestamp_nanos() fails for dates outside ~1000-9999.
        // Practical timestamps fall well within these bounds, so unwrap is safe.
        time::OffsetDateTime::from_unix_timestamp_nanos(dt.timestamp_nanos_opt().unwrap() as i128)
            .unwrap()
    }

    fn chrono_date_to_time(date: chrono::NaiveDate) -> time::Date {
        use chrono::Datelike;
        time::Date::from_calendar_date(
            date.year(),
            time::Month::try_from(date.month() as u8).unwrap(),
            date.day() as u8,
        )
        .unwrap()
    }

    impl<Tz: chrono::TimeZone> From<std::ops::Range<chrono::DateTime<Tz>>> for DateTimeRange {
        fn from(range: std::ops::Range<chrono::DateTime<Tz>>) -> Self {
            Self {
                start: chrono_datetime_to_time(range.start.to_utc()),
                end: chrono_datetime_to_time(range.end.to_utc()),
            }
        }
    }

    impl<Tz: chrono::TimeZone> From<std::ops::RangeInclusive<chrono::DateTime<Tz>>> for DateTimeRange {
        fn from(range: std::ops::RangeInclusive<chrono::DateTime<Tz>>) -> Self {
            Self {
                start: chrono_datetime_to_time(range.start().to_utc()),
                end: chrono_datetime_to_time(range.end().to_utc()) + time::Duration::NANOSECOND,
            }
        }
    }

    impl<Tz: chrono::TimeZone> From<(chrono::DateTime<Tz>, chrono::DateTime<Tz>)> for DateTimeRange {
        fn from(value: (chrono::DateTime<Tz>, chrono::DateTime<Tz>)) -> Self {
            Self {
                start: chrono_datetime_to_time(value.0.to_utc()),
                end: chrono_datetime_to_time(value.1.to_utc()),
            }
        }
    }

    impl<Tz: chrono::TimeZone> From<(chrono::DateTime<Tz>, chrono::Duration)> for DateTimeRange {
        fn from(value: (chrono::DateTime<Tz>, chrono::Duration)) -> Self {
            let start = chrono_datetime_to_time(value.0.to_utc());
            let duration_nanos = value.1.num_nanoseconds().unwrap();
            Self {
                start,
                end: start + time::Duration::nanoseconds(duration_nanos),
            }
        }
    }

    impl From<chrono::NaiveDate> for DateRange {
        fn from(date: chrono::NaiveDate) -> Self {
            Self {
                start: chrono_date_to_time(date),
                end: chrono_date_to_time(date.succ_opt().unwrap()),
            }
        }
    }

    impl From<std::ops::Range<chrono::NaiveDate>> for DateRange {
        fn from(range: std::ops::Range<chrono::NaiveDate>) -> Self {
            Self {
                start: chrono_date_to_time(range.start),
                end: chrono_date_to_time(range.end),
            }
        }
    }

    impl From<std::ops::RangeInclusive<chrono::NaiveDate>> for DateRange {
        fn from(range: std::ops::RangeInclusive<chrono::NaiveDate>) -> Self {
            Self {
                start: chrono_date_to_time(*range.start()),
                end: chrono_date_to_time(range.end().succ_opt().unwrap()),
            }
        }
    }

    impl From<(chrono::NaiveDate, chrono::NaiveDate)> for DateRange {
        fn from(value: (chrono::NaiveDate, chrono::NaiveDate)) -> Self {
            Self {
                start: chrono_date_to_time(value.0),
                end: chrono_date_to_time(value.1),
            }
        }
    }

    impl From<chrono::NaiveDate> for DateTimeRange {
        fn from(date: chrono::NaiveDate) -> Self {
            Self::from(DateRange::from(date))
        }
    }

    impl From<std::ops::Range<chrono::NaiveDate>> for DateTimeRange {
        fn from(range: std::ops::Range<chrono::NaiveDate>) -> Self {
            Self::from(DateRange::from(range))
        }
    }

    impl From<std::ops::RangeInclusive<chrono::NaiveDate>> for DateTimeRange {
        fn from(range: std::ops::RangeInclusive<chrono::NaiveDate>) -> Self {
            Self::from(DateRange::from(range))
        }
    }

    impl From<(chrono::NaiveDate, chrono::NaiveDate)> for DateTimeRange {
        fn from(value: (chrono::NaiveDate, chrono::NaiveDate)) -> Self {
            Self::from(DateRange::from(value))
        }
    }
}

impl TryFrom<(u64, u64)> for DateTimeRange {
    type Error = crate::Error;

    fn try_from(value: (u64, u64)) -> Result<Self, Self::Error> {
        let start = time::OffsetDateTime::from_unix_timestamp_nanos(value.0 as i128)
            .map_err(|e| Error::bad_arg("first UNIX nanos", format!("{e:?}")))?;
        let end = time::OffsetDateTime::from_unix_timestamp_nanos(value.1 as i128)
            .map_err(|e| Error::bad_arg("second UNIX nanos", format!("{e:?}")))?;
        Ok(Self { start, end })
    }
}

trait AddToQuery<T> {
    fn add_to_query(self, param: &T) -> Self;
}

pub(crate) trait AddToForm<T> {
    fn add_to_form(self, param: &T) -> Self;
}

pub(crate) type ReqwestForm = Vec<(&'static str, String)>;

impl AddToQuery<DateRange> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &DateRange) -> Self {
        self.query(&[
            ("start_date", param.start.format(DATE_FORMAT).unwrap()),
            ("end_date", param.end.format(DATE_FORMAT).unwrap()),
        ])
    }
}

impl AddToQuery<DateTimeRange> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &DateTimeRange) -> Self {
        self.query(&[
            ("start", param.start.unix_timestamp_nanos()),
            ("end", param.end.unix_timestamp_nanos()),
        ])
    }
}

impl AddToQuery<Symbols> for reqwest::RequestBuilder {
    fn add_to_query(self, param: &Symbols) -> Self {
        self.query(&[("symbols", param.to_api_string())])
    }
}

impl AddToForm<DateRange> for ReqwestForm {
    fn add_to_form(mut self, param: &DateRange) -> Self {
        self.push(("start_date", param.start.format(DATE_FORMAT).unwrap()));
        self.push(("end_date", param.end.format(DATE_FORMAT).unwrap()));
        self
    }
}

impl AddToForm<DateTimeRange> for ReqwestForm {
    fn add_to_form(mut self, param: &DateTimeRange) -> Self {
        self.push(("start", param.start.unix_timestamp_nanos().to_string()));
        self.push(("end", param.end.unix_timestamp_nanos().to_string()));
        self
    }
}

struct Limit(Option<NonZeroU64>);
impl AddToForm<Limit> for ReqwestForm {
    fn add_to_form(mut self, Limit(limit): &Limit) -> Self {
        if let Some(limit) = limit {
            self.push(("limit", limit.to_string()));
        }
        self
    }
}

#[cfg(test)]
pub(crate) mod test_infra {
    use wiremock::MockServer;

    use crate::HistoricalClient;

    pub const API_KEY: &str = "test-API________________________";

    pub fn client(mock_server: &MockServer) -> HistoricalClient {
        HistoricalClient::builder()
            .base_url(mock_server.uri().parse().unwrap())
            .key(API_KEY)
            .unwrap()
            .build()
            .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use time::macros::{date, datetime};

    #[cfg(feature = "chrono")]
    mod chrono_tests {
        use super::*;
        use chrono::{TimeZone, Utc};

        #[test]
        fn datetime_utc_range() {
            let start = Utc.with_ymd_and_hms(2024, 3, 15, 9, 30, 0).unwrap();
            let end = Utc.with_ymd_and_hms(2024, 3, 15, 16, 0, 0).unwrap();

            let range = DateTimeRange::from(start..end);

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 09:30:00 UTC),
                    datetime!(2024-03-15 16:00:00 UTC)
                ))
            );
        }

        #[test]
        fn datetime_utc_range_inclusive() {
            let start = Utc.with_ymd_and_hms(2024, 3, 15, 9, 30, 0).unwrap();
            let end = Utc.with_ymd_and_hms(2024, 3, 15, 16, 0, 0).unwrap();

            let range = DateTimeRange::from(start..=end);

            // Inclusive end should add 1 nanosecond
            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 09:30:00 UTC),
                    datetime!(2024-03-15 16:00:00.000000001 UTC)
                ))
            );
        }

        #[test]
        fn datetime_utc_tuple() {
            let start = Utc.with_ymd_and_hms(2024, 3, 15, 9, 30, 0).unwrap();
            let end = Utc.with_ymd_and_hms(2024, 3, 15, 16, 0, 0).unwrap();

            let range = DateTimeRange::from((start, end));

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 09:30:00 UTC),
                    datetime!(2024-03-15 16:00:00 UTC)
                ))
            );
        }

        #[test]
        fn datetime_utc_with_duration() {
            let start = Utc.with_ymd_and_hms(2024, 3, 15, 9, 30, 0).unwrap();
            let duration = chrono::Duration::hours(6) + chrono::Duration::minutes(30);

            let range = DateTimeRange::from((start, duration));

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 09:30:00 UTC),
                    datetime!(2024-03-15 16:00:00 UTC)
                ))
            );
        }

        #[test]
        fn naive_date_single() {
            let date = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

            let range = DateTimeRange::from(date);

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 00:00:00 UTC),
                    datetime!(2024-03-16 00:00:00 UTC)
                ))
            );
        }

        #[test]
        fn naive_date_range() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateTimeRange::from(start..end);

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 00:00:00 UTC),
                    datetime!(2024-03-20 00:00:00 UTC)
                ))
            );
        }

        #[test]
        fn naive_date_range_inclusive() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateTimeRange::from(start..=end);

            // Inclusive end date means next day at midnight
            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 00:00:00 UTC),
                    datetime!(2024-03-21 00:00:00 UTC)
                ))
            );
        }

        #[test]
        fn naive_date_tuple() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateTimeRange::from((start, end));

            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 00:00:00 UTC),
                    datetime!(2024-03-20 00:00:00 UTC)
                ))
            );
        }

        #[test]
        fn fixed_offset_datetime_range() {
            use chrono::FixedOffset;

            let offset = FixedOffset::west_opt(4 * 3600).unwrap(); // UTC-4
            let start = offset.with_ymd_and_hms(2024, 3, 15, 9, 30, 0).unwrap();
            let end = offset.with_ymd_and_hms(2024, 3, 15, 16, 0, 0).unwrap();

            let range = DateTimeRange::from(start..end);

            // Should convert to UTC: 09:30 UTC-4 = 13:30 UTC, 16:00 UTC-4 = 20:00 UTC
            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 13:30:00 UTC),
                    datetime!(2024-03-15 20:00:00 UTC)
                ))
            );
        }

        #[test]
        fn fixed_offset_datetime_tuple() {
            use chrono::FixedOffset;

            let offset = FixedOffset::east_opt(5 * 3600 + 30 * 60).unwrap(); // UTC+5:30
            let start = offset.with_ymd_and_hms(2024, 3, 15, 15, 0, 0).unwrap();
            let end = offset.with_ymd_and_hms(2024, 3, 15, 21, 30, 0).unwrap();

            let range = DateTimeRange::from((start, end));

            // 15:00 UTC+5:30 = 09:30 UTC, 21:30 UTC+5:30 = 16:00 UTC
            assert_eq!(
                range,
                DateTimeRange::from((
                    datetime!(2024-03-15 09:30:00 UTC),
                    datetime!(2024-03-15 16:00:00 UTC)
                ))
            );
        }

        #[test]
        fn date_range_from_naive_date_single() {
            let chrono_date = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();

            let range = DateRange::from(chrono_date);

            assert_eq!(
                range,
                DateRange::from((date!(2024 - 03 - 15), date!(2024 - 03 - 16)))
            );
        }

        #[test]
        fn date_range_from_naive_date_range() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateRange::from(start..end);

            assert_eq!(
                range,
                DateRange::from((date!(2024 - 03 - 15), date!(2024 - 03 - 20)))
            );
        }

        #[test]
        fn date_range_from_naive_date_range_inclusive() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateRange::from(start..=end);

            assert_eq!(
                range,
                DateRange::from((date!(2024 - 03 - 15), date!(2024 - 03 - 21)))
            );
        }

        #[test]
        fn date_range_from_naive_date_tuple() {
            let start = chrono::NaiveDate::from_ymd_opt(2024, 3, 15).unwrap();
            let end = chrono::NaiveDate::from_ymd_opt(2024, 3, 20).unwrap();

            let range = DateRange::from((start, end));

            assert_eq!(
                range,
                DateRange::from((date!(2024 - 03 - 15), date!(2024 - 03 - 20)))
            );
        }
    }

    #[test]
    fn date_range_from_lt_day_duration() {
        let target = DateRange::from((date!(2024 - 02 - 16), time::Duration::SECOND));
        assert_eq!(
            target,
            DateRange {
                start: date!(2024 - 02 - 16),
                end: date!(2024 - 02 - 16)
            }
        )
    }

    #[test]
    fn single_date_conversion() {
        let date = date!(2025 - 03 - 27);
        assert_eq!(
            DateRange::from(date),
            DateRange::from((date!(2025 - 03 - 27), date!(2025 - 03 - 28)))
        );
        assert_eq!(
            DateRange::from(date),
            DateRange::from(date!(2025 - 03 - 27)..date!(2025 - 03 - 28))
        );
        assert_eq!(
            DateRange::from(date),
            DateRange::from(date!(2025 - 03 - 27)..=date!(2025 - 03 - 27))
        );
        assert_eq!(
            DateTimeRange::from(date),
            DateTimeRange::from((
                datetime!(2025 - 03 - 27 00:00 UTC),
                datetime!(2025 - 03 - 28 00:00 UTC)
            ))
        );
    }

    #[test]
    fn range_equivalency() {
        let date_range = DateRange::from((date!(2025 - 03 - 27), date!(2025 - 04 - 10)));
        assert_eq!(
            date_range,
            DateRange::from(DateTimeRange::from(date_range.clone()))
        );
    }

    #[test]
    fn dt_offset_to_date_range() {
        assert_eq!(
            DateRange::from(DateTimeRange::from((
                datetime!(2025-03-27 21:00 -4),
                datetime!(2025-03-28 20:00 -4)
            ))),
            DateRange::from((date!(2025 - 03 - 28), date!(2025 - 03 - 29)))
        );
        assert_eq!(
            DateRange::from(DateTimeRange::from((
                datetime!(2025-03-27 21:00 -4),
                datetime!(2025-03-28 20:30 -4)
            ))),
            DateRange::from((date!(2025 - 03 - 28), date!(2025 - 03 - 30)))
        );
    }
}
