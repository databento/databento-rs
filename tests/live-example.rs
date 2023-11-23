//! The example from README.md. Having it here ensures it compiles.
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
            &Subscription::builder()
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
    loop {
        let rec = client.next_record().await?.unwrap();
        if let Some(trade) = rec.get::<TradeMsg>() {
            let symbol = &symbol_map[trade];
            println!("Received trade for {symbol}: {trade:?}");
            break;
        }
        symbol_map.on_record(rec)?;
    }
    Ok(())
}
