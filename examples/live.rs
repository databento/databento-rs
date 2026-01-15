//! The example from README.md. Having it here ensures it compiles.
use std::error::Error;

use databento::{
    dbn::{Dataset, PitSymbolMap, SType, Schema, TradeMsg},
    live::Subscription,
    LiveClient,
};
use dbn::Compression;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_test_writer()
        .init();
    let mut client = LiveClient::builder()
        .key_from_env()?
        .compression(Compression::Zstd)
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
