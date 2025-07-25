# databento-rs

[![build](https://github.com/databento/databento-rs/actions/workflows/build.yaml/badge.svg)](https://github.com/databento/dbn/actions/workflows/build.yaml)
[![Documentation](https://img.shields.io/docsrs/databento)](https://docs.rs/databento/latest/databento/)
[![license](https://img.shields.io/github/license/databento/databento-rs?color=blue)](./LICENSE)
[![Current Crates.io Version](https://img.shields.io/crates/v/databento.svg)](https://crates.io/crates/databento)
[![Slack](https://img.shields.io/badge/join_Slack-community-darkblue.svg?logo=slack)](https://to.dbn.to/slack)

The official Rust client library for [Databento](https://databento.com).
The clients support fast and safe streaming of both real-time and historical market data
through similar interfaces.
The library is built on top of the tokio asynchronous runtime and
[Databento's efficient binary encoding](https://databento.com/docs/standards-and-conventions/databento-binary-encoding).

You can find getting started tutorials, full API method documentation, examples with output on the
[Databento docs site](https://databento.com/docs/?historical=rust&live=rust).

## Installation

To add the crate to an existing project, run the following command:
```sh
cargo add databento
```

### Feature flags

- `historical`: enables the historical client for data older than 24 hours
- `live`: enables the live client for real-time and intraday historical data

By default both features are enabled and the historical client uses OpenSSL for TLS.
To use `rustls`, disable default features for both the databento crate and [reqwest](https://github.com/seanmonstar/reqwest).
```toml
databento = { features = ["historical"], default-features = false }
reqwest = { features = ["rustls-tls"], default-features = false }
```

## Usage

### Historical

Here is a simple program that fetches 10 minutes worth of historical trades for E-mini S&P 500 futures from CME Globex:
```rust no_run
use std::error::Error;

use databento::{
    dbn::{decode::DbnMetadata, Dataset, SType, Schema, TradeMsg},
    historical::timeseries::GetRangeParams,
    HistoricalClient,
};
use time::macros::{date, datetime};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = HistoricalClient::builder().key_from_env()?.build()?;
    let mut decoder = client
        .timeseries()
        .get_range(
            &GetRangeParams::builder()
                .dataset(Dataset::GlbxMdp3)
                .date_time_range((
                    datetime!(2022-06-10 14:30 UTC),
                    datetime!(2022-06-10 14:40 UTC),
                ))
                .symbols("ES.FUT")
                .stype_in(SType::Parent)
                .schema(Schema::Trades)
                .build(),
        )
        .await?;
    let symbol_map = decoder
        .metadata()
        .symbol_map_for_date(date!(2022 - 06 - 10))?;
    while let Some(trade) = decoder.decode_record::<TradeMsg>().await? {
        let symbol = &symbol_map[trade];
        println!("Received trade for {symbol}: {trade:?}");
    }
    Ok(())
}
```

To run this program, set the `DATABENTO_API_KEY` environment variable with an API key and run `cargo bin --example historical`.

### Live

Real-time and intraday replay is provided through the Live clients.
Here is a simple program that fetches the next E-mini S&P 500 futures trade:

```rust no_run
use std::error::Error;

use databento::{
    dbn::{Dataset, PitSymbolMap, SType, Schema, TradeMsg},
    live::Subscription,
    LiveClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = LiveClient::builder()
        .key_from_env()?
        .dataset(Dataset::GlbxMdp3)
        .build()
        .await?;
    client
        .subscribe(
            Subscription::builder()
                .symbols("ES.FUT")
                .schema(Schema::Trades)
                .stype_in(SType::Parent)
                .build(),
        )
        .await
        .unwrap();
    client.start().await?;

    let mut symbol_map = PitSymbolMap::new();
    // Get the next trade
    while let Some(rec) = client.next_record().await? {
        if let Some(trade) = rec.get::<TradeMsg>() {
            let symbol = &symbol_map[trade];
            println!("Received trade for {symbol}: {trade:?}");
            break;
        }
        symbol_map.on_record(rec)?;
    }
    Ok(())
}
```

To run this program, set the `DATABENTO_API_KEY` environment variable with an API key and run `cargo run --example live`

## License

Distributed under the [Apache 2.0 License](https://www.apache.org/licenses/LICENSE-2.0.html).
