use std::error::Error;

use databento::{
    reference::{adjustment, corporate, security, Country, Event},
    ReferenceClient,
};
use time::macros::date;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::FmtSubscriber::builder()
        .with_test_writer()
        .init();
    let mut client = ReferenceClient::builder().key_from_env()?.build()?;

    let actions = client
        .corporate_actions()
        .get_range(
            &corporate::GetRangeParams::builder()
                .symbols("AAPL")
                .start(date!(2024 - 01 - 01))
                .events([Event::Shoch])
                .countries([Country::Us])
                .build(),
        )
        .await?;
    println!("{actions:?}");

    let adjustments = client
        .adjustment_factors()
        .get_range(
            &adjustment::GetRangeParams::builder()
                .symbols("MSFT")
                .start(date!(2024 - 01 - 01))
                .end(date!(2025 - 01 - 01))
                .countries([Country::Us])
                .build(),
        )
        .await?;
    println!("{adjustments:?}");

    let last_sec_master = client
        .security_master()
        .get_last(
            &security::GetLastParams::builder()
                .symbols("AAPL")
                .countries([Country::Us])
                .build(),
        )
        .await?;
    println!("{last_sec_master:?}");

    let sec_master = client
        .security_master()
        .get_range(
            &security::GetRangeParams::builder()
                .symbols("AAPL")
                .countries([Country::Us])
                .start(date!(2005 - 01 - 01))
                .build(),
        )
        .await?;
    println!("{sec_master:?}");

    Ok(())
}
