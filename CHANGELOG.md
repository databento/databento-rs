# Changelog

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
- Initial release with support for historial and live data
