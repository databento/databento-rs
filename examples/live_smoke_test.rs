use clap::Parser;
use databento::{
    dbn::{Dataset, ErrorMsg, MboMsg, RType, Record, SType, Schema},
    live::Subscription,
    LiveClient,
};
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

#[derive(Debug, Parser)]
#[clap(name = "Rust client", version, about)]
pub struct Args {
    #[clap(help = "Gateway address", long)]
    pub gateway: String,

    #[clap(help = "Gateway port", long, default_value = "13000")]
    pub port: u16,

    #[clap(help = "API Key env var", long, default_value = "DATABENTO_API_KEY")]
    pub api_key_env_var: String,

    #[clap(help = "Dataset", long)]
    pub dataset: Dataset,

    #[clap(help = "Schema", long)]
    pub schema: Schema,

    #[clap(help = "SType", long)]
    pub stype: SType,

    #[clap(help = "Symbols", long)]
    pub symbols: String,

    #[clap(help = "Start time (rfc-3339)", long, default_value=None)]
    pub start: Option<String>,

    #[clap(help = "Use snapshot", long, action)]
    pub use_snapshot: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_test_writer()
        .init();
    let args = Args::parse();

    let client = LiveClient::builder()
        .addr((args.gateway.as_str(), args.port))
        .await?
        .key(std::env::var(args.api_key_env_var.clone())?)?
        .dataset(args.dataset)
        .build()
        .await?;

    if args.use_snapshot {
        run_with_snapshot(args, client).await
    } else {
        run(args, client).await
    }
}

async fn run(args: Args, mut client: LiveClient) -> anyhow::Result<()> {
    let start = if let Some(start_time) = &args.start {
        if let Ok(ts) = start_time.parse::<u64>() {
            Some(OffsetDateTime::from_unix_timestamp_nanos(ts as i128)?)
        } else if let Ok(s) = OffsetDateTime::parse(start_time.as_str(), &Rfc3339) {
            Some(s)
        } else {
            return Err(anyhow::format_err!(
                "Timestamp {start_time} is neither nanoseconds from epoch nor in RFC 3339 format",
            ));
        }
    } else {
        None
    };

    let builder = Subscription::builder()
        .schema(args.schema)
        .symbols(args.symbols.clone())
        .stype_in(args.stype);

    let subscription = if let Some(s) = start {
        builder.start(s).build()
    } else {
        builder.build()
    };

    client.subscribe(subscription).await?;

    // For start != 0 we stop at SymbolMappingMsg so that the tests can be run outside trading hours
    let expected_rtype: RType = if start
        .is_some_and(|s| s == OffsetDateTime::UNIX_EPOCH || args.stype == SType::InstrumentId)
    {
        args.schema.into()
    } else {
        RType::SymbolMapping
    };

    client.start().await?;

    println!("Starting client....");

    while let Some(record) = client.next_record().await? {
        if record.header().rtype == expected_rtype as u8 {
            println!("Received expected record {record:?}");
            break;
        } else if let Some(msg) = record.get::<ErrorMsg>() {
            // Unwrap because LSG should always return valid UTF-8
            panic!("Received error: {}", msg.err().unwrap());
        }
    }

    client.close().await?;

    println!("Finished client");

    Ok(())
}

async fn run_with_snapshot(args: Args, mut client: LiveClient) -> anyhow::Result<()> {
    client
        .subscribe(
            Subscription::builder()
                .schema(args.schema)
                .symbols(args.symbols)
                .stype_in(args.stype)
                .use_snapshot()
                .build(),
        )
        .await?;

    client.start().await?;

    println!("Starting client....");

    let mut received_snapshot_record = false;

    while let Some(record) = client.next_record().await? {
        if let Some(msg) = record.get::<MboMsg>() {
            if msg.flags.is_snapshot() {
                received_snapshot_record = true;
            } else {
                println!("Received expected record {record:?}");
                break;
            }
        } else if let Some(msg) = record.get::<ErrorMsg>() {
            // Unwrap because LSG should always return valid UTF-8
            panic!("Received error: {}", msg.err().unwrap());
        }
    }

    client.close().await?;

    println!("Finished client");

    assert!(received_snapshot_record, "Did not receive snapshot record");

    Ok(())
}
