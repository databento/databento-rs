# Changelog

## 0.32.0 - 2025-08-12

### Enhancements
- Upgraded DBN version to 0.39.1:
  - Added `PUBLISHER_SPECIFIC` flag
  - Improved JSON encoding performance
  - Added a `Default` implementation for `SecurityUpdateAction`

### Breaking changes
- Removed unused `Received` variant from `JobState` enum

## 0.31.0 - 2025-07-30

### Enhancements
- Changed `timeseries.get_range_to_file()` to return a concrete type instead of an impl
  trait type
- Upgraded DBN version to 0.39.0:
  - Added `side()` and `unpaired_side()` methods to `ImbalanceMsg` that convert the fields
    of the same name to the `Side` enum
  - Added `pretty_auction_time` property in Python for `ImbalanceMsg`
  - Added `Default` implementation for `StatUpdateAction`
  - Added warnings to the floating-point getter methods' docstrings
  - Added `action` and `ts_in_delta` getters to `BboMsg`
  - Added `ts_recv` getter to `StatusMsg`
  - Added missing floating-point price getters to `InstrumentDefMsg` record types from all
    DBN versions
  - Added more floating-point price getters to `ImbalanceMsg`
  - Added floating-point price getter to `StatMsg` and `v1::StatMsg`

### Breaking changes
- Breaking changes from DBN:
  - Changed `SystemMsg::code()` and `ErrorMsg::code()` methods to return a `Result`
    instead of an `Option` to be consistent with other enum conversion methods
  - Changed `auction_time` field in `ImbalanceMsg` to be formatted as a timestamp

### Bug fixes
- Removed unused `S3` and `Disk` variants from `Delivery` enum

## 0.30.0 - 2025-07-22

### Breaking changes
- Upgraded DBN version to 0.38.0:
  - Renamed `Compression::ZStd` to `Zstd` for consistency

## 0.29.0 - 2025-07-15

### Breaking changes
- Upgraded DBN version to 0.37.0:
  - Renamed the following Venue, Dataset, and Publishers:
      - `XEER` to `XEEE`
      - `XEER.EOBI` to `XEEE.EOBI`
      - `XEER.EOBI.XEER` to `XEEE.EOBI.XEEE`
      - `XEER.EOBI.XOFF` to `XEEE.EOBI.XOFF`
  - Renamed `CMBP1` constant to `CMBP_1` for consistency with `MBP_1` and `MBP_10`

### Bug fixes
- Removed `Intraday` variant from `DatasetCondition` enum

## 0.28.0 - 2025-07-01

### Enhancements
- Added operating system info to the user agent to aid troubleshooting
- Standardized `client` info sent by live clients to match historical
- Added methods to the client builders to extend the user agents with a custom string

### Deprecations
- Deprecated `Historical::with_url()`: use the builder to override the base URL
- Deprecated the `upgrade_policy` parameters for `timseries().get_range()` and
  `timeseries().get_range_to_file()`: use the `Historical` client parameter
  instead

## 0.27.1 - 2025-06-25

### Enhancements
- Added `range_by_schema` field to `DatasetRange` struct

### Bug fixes
- Changed type of `last_modified_date` in `DatasetConditionDetail` to
  `Option<time::Date>` to support missing dates

## 0.27.0 - 2025-06-10

### Enhancements
- Made the buffer size used by the live client when reading from the TCP socket
  configurable through the `LiveBuilder::buffer_size()` method
- Added support for using `rustls` without pulling in OpenSSL. `reqwest` with OpenSSL is
  still the default
- Upgraded DBN version to 0.36.0:
  - Added support for width, fill, and padding when formatting `pretty::Ts`
  - Added support for sign, precision, width, fill, and padding when formatting
    `pretty::Px`
  - Optimized pretty formatting of prices and timestamps

### Breaking changes
- Changed type of `split_duration` to `Option<SplitDuration>` to support setting no
  split duration
- Breaking changes from DBN:
  - Moved core async decoding and encoding functionality to new traits to
    match the sync interface and present a standardized interface
    - Decoding: `AsyncDecodeRecordRef` and `AsyncDecodeRecord`
    - Encoding: `AsyncEncodeRecord`, `AsyncEncodeRecordRef`, and
      `AsyncEncodeRecordTextExt`

### Deprecations
- Deprecated `LiveClient::connect` and `LiveClient::connect_with_addr` methods in favor
  of using the builder so additional optional parameters can be added without a breaking
  change

### Bug fixes
- Fixed bug with deserializing `null` `split_duration` in historical
  `batch().list_jobs()`

## 0.26.2 - 2025-06-03

### Enhancements
- Improved performance of live client by removing redundant state
- Upgraded DBN version to 0.35.1

### Bug fixes
- Fixed handling of `VersionUpgradePolicy` in `timeseries().get_range()` and
  `get_range_to_file()`
- Bug fixes from DBN:
  - Fixed behavior where encoding metadata could lower the `version`
  - Changed `DbnFsm::data()` to exclude all processed data
  - Fixed `Metadata::upgrade()` behavior with `UpgradeToV2`

## 0.26.1 - 2025-05-30

### Bug fixes
- Fixed handling of `VersionUpgradePolicy` in live client
- Fixed default upgrade policies to `UpgradeToV3` to match announcement for
  version 0.26.0

## 0.26.0 - 2025-05-28

This version marks the release of DBN version 3 (DBNv3), which is the new default.
API methods and decoders support decoding all versions of DBN, but now default to
upgrading data to version 3.

### Enhancements - Added `From<DatasetRange>` conversion for `DateTimeRange`
- Added `is_last` field to live subscription requests which will be used to improve the
  handling of split subscription requests
- Upgraded DBN version to 0.35.0:
  - Version 1 and 2 structs can be converted to version 3 structs with the `From` trait
  - Implemented conversion from `RecordRef` to `IoSlice` for use with
    `Write::write_vectored`

### Breaking changes
- Breaking changes from DBN:
  - Definition schema:
    - Updated `InstrumentDefMsg` with new `leg_` fields to support multi-leg strategy
      definitions.
    - Expanded `asset` to 11 bytes and `ASSET_CSTR_LEN` to match
    - Expanded `raw_instrument_id` to 64 bits to support more venues. Like other 64-bit
      integer fields, its value will now be quoted in JSON
    - Removed `trading_reference_date`, `trading_reference_price`, and
      `settl_price_type` fields which will be normalized in the statistics schema
    - Removed `md_security_trading_status` better served by the status schema
  - Statistics schema:
    - Updated `StatMsg` has an expanded 64-bit `quantity` field. Like other 64-bit
      integer fields, its value will now be quoted in JSON
    - The previous `StatMsg` has been moved to `v2::StatMsg` or `StatMsgV2`
  - Changed the default `VersionUpgradePolicy` to `UpgradeToV3`
  - Updated the minimum supported `tokio` version to 1.38, which was released one year ago

## 0.25.0 - 2025-05-13

### Enhancements
- Increased live subscription symbol chunking size
- Upgraded DBN version to 0.34.0:
  - Added a `v3::StatMsg` record with an expanded 64-bit `quantity` field
  - Added `with_compression_level` methods to `DynWriter`, `AsyncDynWriter`, and
    `AsyncDynBufWriter`
  - Added `DBN_VERSION` constants to each version module: `v1`, `v2`, and `v3`
  - Added `UNDEF_STAT_QUANTITY` constants to each version module
  - Added statistics compatibility trait `StatRec` for generalizing across different
    versions of the statistics record
  - Added `AsRef<[u8]>` implementations for `RecordEnum` and `RecordRefEnum`
  - Added new off-market publishers for Eurex, and European Energy Exchange (EEX)

### Breaking changes
- From DBN:
  - Made `Record` a subtrait of `AsRef<[u8]>` as all records should be convertible to
    bytes

## 0.24.0 - 2025-04-22

### Enhancements
- Upgraded DBN version to 0.33.0:
  - Added `SystemCode` and `ErrorCode` enums to indicate types of system and error
    messages
  - Added `code()` methods to `SystemMsg` and `ErrorMsg` to retrieve the enum value if
    one exists and equivalent properties in Python
  - Converting a `v1::SystemMsg` to a `v2::SystemMsg` now sets to `code` to the
    heartbeat value
  - Added `ASSET_CSTR_LEN` constants for the size of `asset` field in `InstrumentDefMsg`
    in different DBN versions
  - Added `encode_record_with_sym()`  method to `AsyncJsonEncoder` which encodes a
    record along with its text symbol to match the sync encoder

### Breaking changes
- Breaking changes from DBN:
  - Added `code` parameter to `SystemCode::new()` and `ErrorMsg::new()`
  - Updated the `rtype_dispatch` and `schema_dispatch` macro invocations to look more
    like function invocation
  - Increased the size of `asset` field in `v3::InstrumentDefMsg` from 7 to 11. The
    `InstrumentDefMsgV3` message size remains 520 bytes.

## 0.23.0 - 2025-04-15

### Enhancements
- Added `subscriptions` to `LiveClient` `Debug` implementation
- Upgraded DBN version to 0.32.0:
  - Added `SystemCode` and `ErrorCode` enums to indicate types of system and error
    messages
  - Added `code()` methods to `SystemMsg` and `ErrorMsg` to retrieve the enum value if
    one exists and equivalent properties in Python
  - Converting a `v1::SystemMsg` to a `v2::SystemMsg` now sets to `code` to the heartbeat
    value
  - Added `Ord` and `PartialOrd` implementations for all enums and `FlagSet` to allow
    for use in ordered containers like `BTreeMap`
  - Added `decode_records()` method to `AsyncDbnDecoder` and `AsyncDbnRecordDecoder`
    which is similar to the sync decoder methods of the same name
  - Upgraded `pyo3` version to 0.24.1
  - Upgraded `time` version to 0.3.41

### Breaking changes
- Added new `id` field to live `Subscription`, which will be used for improved error
  messages
- Added new `id` parameter to `live::protocol::SubRequest::new()` method
- Breaking changes from DBN:
  - Added `code` parameter to `SystemCode::new()` and `ErrorMsg::new()`
  - Updated the `rtype_dispatch` and `schema_dispatch` macro invocations to look more like
    function invocation
  - Removed deprecated `dataset` module. The top-level `Dataset` enum and its `const` `as_str()`
    method provide the same functionality for all datasets
  - Removed deprecated `SymbolIndex::get_for_rec_ref()` method

## 0.22.0 - 2025-04-01

### Enhancements
- Added an implementation `From<Date>` for `DateRange` and `DateTimeRange` to make it
  simpler to request a single full day's worth of data
- Added conversions between `DateRange` and `DateTimeRange`
- Added conversions from `timeseries::GetRangeParams`, `timeseries::GetRangeToFileParams`,
  and `dbn::Metadata` to `symbology::ResolveParams`
- Upgraded DBN version to 0.31.0:
  - Added support for mapping symbols from instrument definitions to `PitSymbolMap`
    with a new `on_instrument_def()` method
  - Added instrument definition compatibility trait `InstrumentDefRec` for generalizing
    across different versions of the instrument definition record
  - Added `Ord` and `PartialOrd` implementations for all enums and `FlagSet` to allow
    for use in ordered containers like `BTreeMap`
  - Added `decode_records()` method to `AsyncDbnDecoder` and `AsyncDbnRecordDecoder`
    which is similar to the sync decoder methods of the same name
  - Removed deprecated `dataset` module. The top-level `Dataset` enum and its `const` `as_str()`
    method provide the same functionality for all datasets
  - Removed deprecated `SymbolIndex::get_for_rec_ref()` method

## 0.21.0 - 2025-03-18

### Enhancements
- Improved error when calling `LiveClient::next_record()` on an instance that hasn't
  been started
- Improved error when calling `LiveClient::start()` on an instance that has already
  been started
- Upgraded DBN version to 0.29.0:
  - Added new venues, datasets, and publishers for ICE Futures US, ICE Europe Financials
    products, Eurex, and European Energy Exchange (EEX)
  - Added new `SkipBytes` and `AsyncSkipBytes` traits which are a subset of the `Seek`
    and `AsyncSeek` traits respectively, only supporting seeking forward from the current
    position
  - Deprecated `AsyncRecordDecoder::get_mut()` and `AsyncDecoder::get_mut()` as modifying
    the inner reader after decoding any records could lead to a corrupted stream and
    decoding errors

## 0.20.0 - 2025-02-12

### Enhancements
- Added `LiveClient::reconnect()` and `LiveClient::resubscribe()` methods to make it easier
  to resume a live session after losing the connection to the live gateway
- Added `subscriptions()` and `subscriptions_mut()` getters to `LiveClient` for getting all
  active subscriptions
- Added `shutdown()` method to `live::Protocol` to clean up the active session
- Downgraded to tracing span level on `LiveClient::next_record()` to "debug" to reduce
  performance impact
- Added `From<&[&str]>` and `From<[str; N]>` implementations for `Symbols`

### Breaking changes
- Changed `LiveClient::close()` to take `&mut self` rather than an owned value to `self` now
  that clients can be reused through the `reconnect()` method
- Changed `LiveClient::subscribe()` to take a `Subscription` parameter rather than a
  `&Subscription` because it will now store the `Subscription` struct internally
- Upgraded DBN version to 0.28.0:
  - Added `CommoditySpot` `InstrumentClass` variant and made `InstrumentClass`
    non-exhaustive to allow future additions without breaking changes

## 0.19.0 - 2025-01-21

### Enhancements
- Upgraded DBN version to 0.27.0:
  - Updated enumerations for unreleased US equities datasets and publishers
  - Added new venue `EQUS` for consolidated US equities
  - Added new dataset `EQUS.MINI` and new publishers `EQUS.MINI.EQUS` and
    `XNYS.TRADES.EQUS`

### Bug fixes
- Changed historical metadata methods with `symbols` parameter to use a `POST` request
  to allow for requesting supported maximum of 2000 symbols

## 0.18.0 - 2025-01-08

### Enhancements
- Upgraded DBN version to 0.26.0:
  - Added `v3` namespace in preparation for future DBN version 3 release. DBN version 2
    remains the current and default version
  - Added `v3::InstrumentDefMsg` record with new fields to support normalizing multi-leg
    strategy definitions
    - Removal of statistics-schema related fields `trading_reference_price`,
      `trading_reference_date`, and `settl_price_type`
    - Removal of the status-schema related field `md_security_trading_status`
  - Added initial support for merging DBN:
    - Decoding streams: `MergeDecoder` and `MergeRecordDecoder` structs
    - Metadata: `MergeDecoder` struct and `Metadata::merge()` method
    - In the CLI: specify more than one input file to initiate a merge
  - Eliminated `unsafe` in `From` implementations for record structs from different
    versions

## 0.17.0 - 2024-12-17

### Enhancements
- Upgraded DBN version to 0.25.0:
  - Added `v1` and `v2` namespaces in DBN to allow unambiguously referring to the record
    types for a given DBN version regardless of whether the record type has changed
  - Changed `dataset()` method on `MetadataBuilder` to accept an `impl ToString` so now
    `Dataset` and `&str` can be passed directly
  - Changed async DBN decoding to return `Ok(None)` when an incomplete record remains in
    the stream. This matches the existing behavior of sync DBN decoding
- Upgraded `thiserror` version to 2.0

### Breaking changes
- Removed deprecated `Packaging` enum and `packaging` field that's no longer supported
  by the API
- As part of the DBN version upgrade:
  - `VersionUpgradePolicy::Upgrade` was renamed to `UpgradeToV2`
  - Changed async DBN decoding to return `Ok(None)` when an incomplete record remains in
    the stream

## 0.16.0 - 2024-11-12

#### Enhancements
- Upgraded DBN version to 0.23.1:
  - Added floating-point getters for price fields
  - Added new IntelligentCross venues `ASPN`, `ASMT`, and `ASPI`
  - Upgraded `thiserror` version to 2.0

#### Deprecations
- Deprecated `Packaging` enum and `packaging` field on `SubmitJobParams` and `BatchJob`.
  These will be removed in a future version. All files from a batch job can be
  downloaded with the `batch().download()` method on the historical client

## 0.15.0 - 2024-10-22

#### Enhancements
- Upgraded DBN version to 0.23.0:
  - Added new `None` `Action` variant that will be gradually rolled out
    to historical and live `GLBX.MDP3` data
  - Added consistent escaping of non-printable and non-ASCII values when text encoding
    `c_char` fields
  - Implemented `Default` for `Action` and `Side`
  - Implemented missing `Serialize` for (with `serde` feature enabled) for `Venue`,
    `Dataset`, `Publisher`, `Compression`, `SType`, `Schema`, and `Encoding`

## 0.14.1 - 2024-10-08

#### Enhancements
- Upgraded DBN version to 0.22.1:
  - Fixed buffer overrun
  - Combined `_reserved3` and `reserved4` fields in `CbboMsg`

## 0.14.0 - 2024-10-01

#### Enhancements
- Made several previously internal functions public to allow advanced users more
  customization and piecemeal usage of the live API:
  - `ApiKey`
  - `Symbols::to_chunked_api_string()`
  - `live::protocol` module containing implementations of the raw API messages
- Changed from `log` crate to `tracing` for better diagnostics

## 0.13.0 - 2024-09-25

#### Enhancements
- Upgraded DBN version to 0.21.0 for:
  - Changed the layout of `CbboMsg` to better match `BboMsg`
  - Renamed `Schema::Cbbo` to `Schema::Cmbp1`
- Upgraded `typed-builder` version to 0.20

#### Deprecations
- Deprecated `Packaging::Tar`. Users should switch to `Packaging::Zip`. This variant
  will be removed in a future version when it is no longer supported by the API

## 0.12.1 - 2024-08-27

#### Enhancements
- Added `Intraday` variant to `DatasetCondition` in preparation for intraday data being
  available from the historical API
- Upgraded DBN version to 0.20.1 for new publisher values for `XCIS.BBOTRADES` and
  `XNYS.BBOTRADES`

## 0.12.0 - 2024-07-30

#### Breaking changes
- Upgraded DBN version to 0.20.0:
  - Renamed `SType::Nasdaq` variant to `SType::NasdaqSymbol`
  - Renamed `SType::Cms` variant to `SType::CmsSymbol`

## 0.11.4 - 2024-07-16

#### Enhancements
- Upgraded DBN version to 0.19.1 with fixes for `BBOMsg` record struct

## 0.11.3 - 2024-07-09

#### Enhancements
- Upgraded DBN version to 0.19.0 with new `BBOMsg` record struct

## 0.11.2 - 2024-06-25

#### Enhancements
- Added `historical::timeseries::get_range_to_file` method to persist the data stream to
  a given path before returning an `AsyncDbnDecoder`
- Upgraded DBN version to 0.18.2

## 0.11.1 - 2024-06-11

#### Enhancements
- Added getter for `heartbeat_interval` to `LiveClient`

#### Bug fixes
- Fixed potential incorrect DNS resolution when overriding the live gateway address
  with `live::Builder::addr`

## 0.11.0 - 2024-06-04

#### Enhancements
- Added configurable `heartbeat_interval` parameter for live client that determines the
  timeout before heartbeat `SystemMsg` records will be sent. It can be configured via
  the `heartbeat_interval` and `heartbeat_interval_s` methods of the
  `live::ClientBuilder`
- Added `addr` function to `live::ClientBuilder` for configuring a custom gateway
  address without using `LiveClient::connect_with_addr` directly
- Upgraded DBN version to 0.18.1

#### Breaking changes
- Added `heartbeat_interval` parameter to `LiveClient::connect` and
  `LiveClient::connect_with_addr`
- Removed deprecated `start_date` and `end_date` fields from `DatasetRange` struct

## 0.10.0 - 2024-05-22

#### Enhancements
- Added `use_snapshot` attribute to `Subscription`, defaults to false
- Upgraded reqwest version to 0.12

#### Breaking changes
- Upgraded DBN version to 0.18.0
  - Changed type of `flags` in `MboMsg`, `TradeMsg`, `Mbp1Msg`, `Mbp10Msg`, and `CbboMsg`
    from `u8` to a new `FlagSet` type with predicate methods for the various bit flags
    as well as setters. The `u8` value can still be obtained by calling the `raw()` method
    - Improved `Debug` formatting
  - Switched `DecodeStream` from `streaming_iterator` crate to `fallible_streaming_iterator`
    to allow better notification of errors
  - Changed default value for `stype_in` and `stype_out` in `SymbolMappingMsg` to
    `u8::MAX` to match C++ client and to reflect an unknown value. This also changes the
    value of these fields when upgrading a `SymbolMappingMsgV1` to DBNv2

## 0.9.1 - 2024-05-15

#### Bug fixes
- Fixed build when only `live` feature is enabled

## 0.9.0 - 2024-05-14

#### Enhancements
- Added `start` and `end` fields to the `DatasetRange` struct which provide time
  resolution and an exclusive end date
- Upgraded DBN version to 0.17.1

#### Deprecations
- The `start_date` and `end_date` fields of the `DatasetRange` struct are deprecated and
  will be removed in a future release

## 0.8.0 - 2024-04-01

#### Enhancements
- Upgraded DBN version to 0.17.0
  - Added new record types and schema variants for consolidated BBO and subsampled BBO
  - Added `Volatility` and `Delta` `StatType` variants

#### Breaking changes
- Removed previously-deprecated `live::SymbolMap`. Please use
  `databento::dbn::PitSymbolMap` instead

## 0.7.1 - 2024-03-05

#### Enhancements
- Improve error handling when a historical HTTP error response is not in the
  expected JSON format

## 0.7.0 - 2024-03-01

#### Enhancements
- Document cancellation safety of `LiveClient` methods (credit: @yongqli)
- Document `live::Subscription::start` is based on `ts_event`
- Allow constructing a `DateRange` and `DateTimeRange` with an `end` based on a
  `time::Duration`
- Implemented `Debug` for `LiveClient`, `live::ClientBuilder`, `HistoricalClient`,
  `historical::ClientBuilder`, `BatchClient`, `MetadataClient`, `SymbologyClient`, and
  `TimeseriesClient`
- Derived `Clone` for `live::ClientBuilder` and `historical::ClientBuilder`
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
- Upgraded DBN version to 0.15.0
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
- Upgraded DBN version to 0.10.2 for easier historical symbology

## 0.2.1 - 2023-08-25

#### Enhancements
- Upgraded DBN version to 0.9.0 for publisher improvements to support OPRA

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
