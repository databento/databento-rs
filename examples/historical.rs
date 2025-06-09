//! The example from README.md. Having it here ensures it compiles.
use std::error::Error;

use databento::{
    dbn::{Dataset, SType, Schema, TradeMsg},
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
