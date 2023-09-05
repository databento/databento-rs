# Changelog

## 0.3.0 - TBD

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
