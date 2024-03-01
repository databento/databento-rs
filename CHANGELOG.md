# Changelog

## 0.7.0 - 2024-03-01

#### Enhancements
- Document cancellation safety of `LiveClient` methods (credit: @yongqli)
- Document `live::Subscription::start` is based on `ts_event`
- Allow constructing a `DateRange` and `DateTimeRange` with an `end` based on a
  `time::Duration`
- Implemented `Debug` for `LiveClient`, `LiveClientBuilder`, `HistoricalClient`,
  `HistoricalClientBuilder`, `BatchClient`, `MetadataClient`, `SymbologyClient`, and
  `TimeseriesClient`
- Derived `Clone` for `LiveClientBuilder` and `HistoricalClientBuilder`
- Added `ApiKey` type for safely deriving `Debug` for types containing an API key

#### Breaking changes
- Changed default `upgrade_policy` in `LiveBuilder` and `GetRangeParams` to `Upgrade` so
  by default the primary record types can always be used
- Simplified `DateRange` and `DateTimeRange` by removing `FwdFill` variant that didn't
  work correctly
- Upgraded DBN version to 0.16.0
  - Updated `StatusMsg` in preparation for status schema release
  - Fixed handling of `ts_out` when upgrading DBNv1 records to version 2
  - Fixed handling of `ErrorMsgV1` and `SystemMsgV1` in `rtype` dispatch macros

## 0.6.0 - 2024-01-16

#### Enhancements
- Relaxed version requirements for `tokio`, `tokio-util`, and `thiserror`

#### Breaking changes
- Updated DBN version to 0.15.0
  - Added support for larger `SystemMsg` and `ErrorMsg` records
  - Improved `Debug` implementations for records and `RecordRef`
  - Improved panic messages for `RecordRef::get`
- Upgraded `typed-builder` to 0.18

#### Bug fixes
- Fixed documentation for `end` in `DateRange::Closed` and `DateTimeRange::Closed`

## 0.5.0 - 2023-11-23

This release adds support for DBN v2.

DBN v2 delivers improvements to the `Metadata` header symbology, new `stype_in` and `stype_out`
fields for `SymbolMappingMsg`, and extends the symbol field length for `SymbolMappingMsg` and
`InstrumentDefMsg`. The entire change notes are available [here](https://github.com/databento/dbn/releases/tag/v0.14.0).
Users who wish to convert DBN v1 files to v2 can use the `dbn-cli` tool available in the [databento-dbn](https://github.com/databento/dbn/) crate.
On a future date, the Databento live and historical APIs will stop serving DBN v1.

This release is fully compatible with both DBN v1 and v2, and so should be seamless for most users.

#### Enhancements
- Made `LiveClient::next_record`, `dbn::decode::AsyncDbnDecoder::decode_record` and
  `decode_record_ref`, and `dbn::decode::AsyncRecordDecoder::decode` and `decode_ref`
  cancel safe. This makes them safe to use within a
  [`tokio::select!`](https://docs.rs/tokio/latest/tokio/macro.select.html) statement
- Improved error reporting for `HistoricalClient` when receiving an error from
  Databento's API
- Improved error messages around API keys
- Improved performance of CSV and JSON encoding
- Added support for emitting warnings from historical API response headers, such as for
  future deprecations
- Added `symbol_map` method to the `Resolution` struct returned by `symbology::resolve`
  that returns a `TsSymbolMap`
- Added `PartialEq` and `Eq` implementations for parameter builder classes
- Added `upgrade_policy` setter to the `LiveClient` builder and a getter to the
  `LiveClient`
- Added `upgrade_policy` optional setter to the `timeseries::GetRangeParams` builder

#### Breaking changes
- Upgraded `dbn` to 0.14.2. There are several breaking changes in this release as we
  begin migrating to DBN encoding version 2 (DBNv2) in order to support the ICE
  exchange:
  - Renamed `dbn::InstrumentDefMsg` to `dbn::compat::InstrumentDefMsgV1` and added a
    new `dbn::InstrumentDefMsg` with a longer `raw_symbol` field
  - Renamed `dbn::SymbolMappingMsg` to `dbn::compat::SymbolMappingMsgV1` and added a
    new `dbn::SymbolMappingMsg` with longer symbol fields and new `stype_in` and
    `stype_out` fields
  - Added `symbol_cstr_len` field to `dbn::Metadata`
- Made `Error` non-exhaustive, meaning it no longer be exhaustively matched against, and
  new variants can be added in the future without a breaking change
- Added an `upgrade_policy` parameter to `LiveClient::connect` and `connect_with_addr`.
  The builder provides a more stable API since new parameters are usually introduced as
  optional

#### Deprecations
- Deprecated `live::SymbolMap` in favor of `databento::dbn::PitSymbolMap`

## 0.4.2 - 2023-10-23

#### Enhancemets
- Upgraded `dbn` to 0.13.0 for improvements to symbology helpers
- Upgraded `tokio` to 1.33
- Upgraded `typed-builder` to 0.17

#### Bug fixes
- Fixed panic in `LiveClient` when gateway returned an auth response without the
  `success` key

## 0.4.1 - 2023-10-06

#### Enhancements
- Added support for changing datetime format used in batch job responses
- Upgraded `dbn` to 0.11.1

## 0.4.0 - 2023-09-21

#### Enhancements
- Added `pretty_px` option for `batch::submit_job`, which formats prices to the correct
  scale using the fixed-precision scalar 1e-9 (available for CSV and JSON text
  encodings)
- Added `pretty_ts` option for `batch::submit_job`, which formats timestamps as ISO 8601
  strings (available for CSV and JSON text encodings)
- Added `map_symbols` option to `batch::submit_job`, which appends the raw symbol to
  every record (available for CSV and JSON text encodings) reducing the need to look at
  the `symbology.json` file
- Added `split_symbols` option for `batch::submit_job`, which will split files by raw
  symbol
- Added `encoding` option to `batch::submit_job` to allow requesting non-DBN encoded
  data through the client
- Added `map_symbols`, `pretty_px`, and `pretty_ts` to `BatchJob` response
- Added default `stype_in` of `RawSymbol` for live subscriptions to match behavior of
  the historical client and the Python client

## 0.3.0 - 2023-09-13

#### Enhancements
- Added `SymbolMap` type to help maintain up-to-date symbol mappings with live data
- Added chunking to handle subscribing to many symbols for the Live client
- Updated DBN version to 0.10.2 for easier historical symbology

## 0.2.1 - 2023-08-25

#### Enhancements
- Updated DBN version to 0.9.0 for publisher improvements to support OPRA

## 0.2.0 - 2023-08-10

#### Breaking changes
- Changed `metadata::list_publishers` to return a `Vec<PublisherDetail>`
- `metadata::list_fields`:
  - Changed return type to `Vec<FieldDetail>`
  - Made `encoding` and `schema` parameters required
  - Removed `dataset` parameter
- `metadata::list_unit_prices`:
  - Changed return type to `Vec<UnitPricesForMode>`
  - Made `dataset` parameter required
  - Removed `mode` and `schema` parameters

## 0.1.0 - 2023-08-02
- Initial release with support for historical and live data
