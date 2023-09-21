//! The example from README.md. Having it here ensures it compiles.
use std::error::Error;

use databento::{
    dbn::{Schema, TradeMsg},
    historical::timeseries::GetRangeParams,
    HistoricalClient, Symbols,
};
use time::macros::datetime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = HistoricalClient::builder().key_from_env()?.build()?;
    let mut decoder = client
        .timeseries()
        .get_range(
            &GetRangeParams::builder()
                .dataset("GLBX.MDP3")
                .date_time_range((
                    datetime!(2022-06-10 14:30 UTC),
                    datetime!(2022-06-10 14:40 UTC),
                ))
                .symbols(Symbols::All)
                .schema(Schema::Trades)
                .build(),
        )
        .await?;
    while let Some(trade) = decoder.decode_record::<TradeMsg>().await? {
        println!("{trade:?}");
    }
    Ok(())
}
