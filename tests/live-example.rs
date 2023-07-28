//! The example from README.md. Having it here ensures it compiles.
use std::{collections::HashMap, error::Error};

use databento::{
    dbn::{
        datasets,
        enums::{SType, Schema},
        record::{SymbolMappingMsg, TradeMsg},
    },
    live::Subscription,
    LiveClient,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut client = LiveClient::builder()
        .key_from_env()?
        .dataset(datasets::GLBX_MDP3)
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

    let mut symbol_mappings = HashMap::new();
    // Get the next trade
    loop {
        let rec = client.next_record().await?.unwrap();
        if rec.has::<TradeMsg>() {
            let trade = rec.get::<TradeMsg>().unwrap();
            let symbol = symbol_mappings.get(&trade.hd.instrument_id).unwrap();
            println!("Received trade for {symbol}: {trade:?}",);
            break;
        } else if rec.has::<SymbolMappingMsg>() {
            let sym_map = rec.get::<SymbolMappingMsg>().unwrap();
            symbol_mappings.insert(
                sym_map.hd.instrument_id,
                sym_map.stype_out_symbol()?.to_owned(),
            );
        }
    }

    Ok(())
}
