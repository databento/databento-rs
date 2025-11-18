//! An example program that splits a DBN file into several DBN files
//! by parent symbol (from the `asset` field in the definitions schema).
use std::collections::HashMap;

use anyhow::Context;
use async_compression::tokio::write::ZstdEncoder;
use databento::{
    dbn::{
        decode::{AsyncDbnDecoder, DbnMetadata},
        encode::{AsyncDbnEncoder, AsyncEncodeRecord, AsyncEncodeRecordRef},
        InstrumentDefMsg, Metadata, Schema, SymbolIndex,
    },
    historical::timeseries::GetRangeParams,
    HistoricalClient,
};
use tokio::fs::File;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_test_writer()
        .init();
    if std::env::args().len() != 3 {
        anyhow::bail!(
            "Invalid number of arguments, expected: split_symbols FILE_PATH OUTPUT_PATTERN"
        );
    }
    let file_path = std::env::args().nth(1).unwrap();
    let output_pattern = std::env::args().nth(2).unwrap();
    if !output_pattern.contains("{parent}") {
        anyhow::bail!("OUTPUT_PATTERN should contain {{parent}}");
    }
    let mut decoder = AsyncDbnDecoder::from_zstd_file(file_path).await?;

    let metadata = decoder.metadata().clone();
    let symbol_map = metadata.symbol_map()?;
    let symbols_to_parent = fetch_symbols_to_parent(&metadata).await?;
    let mut encoders = HashMap::<String, AsyncDbnEncoder<ZstdEncoder<File>>>::new();
    while let Some(rec) = decoder.decode_record_ref().await? {
        let Some(symbol) = symbol_map.get_for_rec(&rec) else {
            eprintln!("Missing mapping for {rec:?}");
            continue;
        };
        let Some(parent) = symbols_to_parent.get(symbol) else {
            eprintln!("Couldn't find parent mapping for {symbol} with {rec:?}");
            continue;
        };
        if let Some(encoder) = encoders.get_mut(parent) {
            encoder.encode_record_ref(rec).await?;
        } else {
            let mut encoder = AsyncDbnEncoder::with_zstd(
                File::create_new(output_pattern.replace("{parent}", parent))
                    .await
                    .with_context(|| format!("creating file for {parent}"))?,
                &metadata,
            )
            .await?;
            encoder.encode_record_ref(rec).await?;
            encoders.insert(parent.clone(), encoder);
        };
    }
    for (parent, mut encoder) in encoders {
        if let Err(e) = encoder.shutdown().await {
            eprintln!("Failed to shutdown encoder for {parent}: {e:?}");
        }
    }

    Ok(())
}

async fn fetch_symbols_to_parent(metadata: &Metadata) -> anyhow::Result<HashMap<String, String>> {
    let mut client = HistoricalClient::builder().key_from_env()?.build()?;
    let end = metadata.end().ok_or_else(|| {
        anyhow::format_err!("Missing end in metadata. This script is intended for historical data")
    })?;
    let mut res = HashMap::new();
    // 2000 is the maximum number of symbols per request
    for chunk in metadata.symbols.chunks(2000) {
        let mut decoder = client
            .timeseries()
            .get_range(
                &GetRangeParams::builder()
                    .dataset(metadata.dataset.clone())
                    .schema(Schema::Definition)
                    .date_time_range(metadata.start()..end)
                    .symbols(Vec::from(chunk))
                    .build(),
            )
            .await?;
        while let Some(def) = decoder.decode_record::<InstrumentDefMsg>().await? {
            res.insert(def.raw_symbol()?.to_owned(), def.asset()?.to_owned());
        }
    }
    Ok(res)
}
