//! The adjustment factors API.

use dbn::{Compression, SType};
use serde::Deserialize;
use time::{Date, OffsetDateTime};
use typed_builder::TypedBuilder;

use crate::{
    deserialize::deserialize_date_time,
    historical::{handle_zstd_jsonl_response, AddToForm},
    reference::{AdjustmentStatus, Country, Currency, End, Event, Frequency, SecurityType, Start},
    DateTimeLike, Symbols,
};

/// A client for the adjustment factors group of Reference API endpoints.
#[derive(Debug)]
pub struct AdjustmentFactorsClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl AdjustmentFactorsClient<'_> {
    /// Requests a new adjustment factor time series from Databento.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_range(
        &mut self,
        params: &GetRangeParams,
    ) -> crate::Result<Vec<AdjustmentFactor>> {
        let form = vec![
            ("stype_in", params.stype_in.to_string()),
            ("symbols", params.symbols.to_api_string()),
            ("compression", Compression::Zstd.to_string()),
        ]
        .add_to_form(&Start(params.start))
        .add_to_form(&End(params.end))
        .add_to_form(&params.countries)
        .add_to_form(&params.security_types);
        let resp = self
            .inner
            .post("adjustment_factors.get_range")?
            .form(&form)
            .send()
            .await?;
        let mut adjustment_factors: Vec<AdjustmentFactor> =
            handle_zstd_jsonl_response(resp).await?;
        adjustment_factors.sort_by_key(|a| a.ex_date);
        Ok(adjustment_factors)
    }
}

/// The parameters for [`AdjustmentFactorsClient::get_range()`]. Use
/// [`GetRangeParams::builder()`] to get a builder type with all the preset defaults.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct GetRangeParams {
    /// The inclusive start time of the request range. Filters on `index`.
    #[builder(setter(transform = |dt: impl DateTimeLike| dt.to_date_time()))]
    pub start: OffsetDateTime,
    /// The exclusive end time of the request range. Filters on `index`.
    ///
    /// If `None`, all data after `start` will be included in the response.
    #[builder(default, setter(transform = |dt: impl DateTimeLike| Some(dt.to_date_time())))]
    pub end: Option<OffsetDateTime>,
    /// The symbols to filter for.
    #[builder(setter(into))]
    pub symbols: Symbols,
    /// The symbology type of the input `symbols`. Defaults to
    /// [`RawSymbol`](SType::RawSymbol).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// An optional list of country codes to filter for. By default all countries are
    /// included.
    #[builder(default, setter(into))]
    pub countries: Vec<Country>,
    /// An optional list of security types to filter for. By default all security types
    /// are included.
    #[builder(default, setter(into))]
    pub security_types: Vec<SecurityType>,
}

/// A record in the adjustment factor response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct AdjustmentFactor {
    /*
     * Identifiers
     */
    /// Security level numerical ID. Can be used to link all multiple listings together.
    pub security_id: String,
    /// Event identifier unique at the event level. Links to the corporate actions `event_id`.
    pub event_id: String,
    /// The event type.
    pub event: Event,

    /*
     * Exchange
     */
    /// The issuer name.
    pub issuer_name: String,
    /// The security type.
    pub security_type: SecurityType,
    /// Exchange code for the primary security.
    pub primary_exchange: Option<String>,
    /// Exchange code for the listing. Equivalent to the MIC but more stable as MIC
    /// might not be available in a timely fashion. Also note that the MIC can change
    /// but the exchange will remain the same.
    pub exchange: Option<String>,
    /// Market Identifier Code (MIC) as an ISO 10383 string.
    pub operating_mic: String,

    /*
     * Symbology
     */
    /// The query input symbol which matched the record.
    pub symbol: Option<String>,
    /// Nasdaq Integrated Platform Suffix convention symbol.
    pub nasdaq_symbol: Option<String>,
    /// Local Code. Usually unique at market level but there are exceptions to this
    /// rule. Either an alpha string, or a number.
    pub local_code: Option<String>,
    /// Resultant local code when applicable/known.
    pub local_code_resulting: Option<String>,
    /// ISIN global level identifier as an ISO 6166 string.
    pub isin: Option<String>,
    /// Resultant ISIN when applicable/known.
    pub isin_resulting: Option<String>,
    /// US domestic CUSIP.
    pub us_code: Option<String>,

    /// The adjustment status.
    pub status: AdjustmentStatus,
    /// Date from which the event is effective.
    pub ex_date: Date,
    /// Adjustment factor to apply.
    pub factor: f64,
    /// Closing price on the `ex_date`.
    pub close: Option<f64>,
    /// Currency for the closing price.
    pub currency: Option<String>,
    /// Market sentiment - the market's reaction to the event. Simply the previous close
    /// divided by today's open. Only correct if factor calculation requires previous close.
    pub sentiment: f64,
    /// Reason/type of event, used to distinguish between different event types.
    pub reason: u32,
    /// The amount of the dividend before any taxes or fees are deducted. This value
    /// represents the total dividend declared by the company.
    pub gross_dividend: Option<f64>,
    /// The currency in which the dividend is paid.
    pub dividend_currency: Option<Currency>,
    /// The frequency at which the dividend is paid.
    pub frequency: Option<Frequency>,
    /// The choice or option number associated with the event, often used when
    /// shareholders are given multiple options for how they would like to receive the
    /// dividend or other corporate action benefit (either cash, or script).
    pub option: u32,
    /// A human-readable description of the event.
    pub detail: String,
    /// The timestamp (UTC) the record was added by Databento.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_created: OffsetDateTime,
}

#[cfg(test)]
mod tests {
    use std::io;

    use reqwest::StatusCode;
    use time::macros::{date, datetime};
    use wiremock::{
        matchers::{basic_auth, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{
        body_contains,
        historical::{test_infra::API_KEY, API_VERSION},
        reference::test_infra::client,
    };

    #[tokio::test]
    async fn test_get_range() {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_test_writer()
            .try_init();
        let start = datetime!(2023- 10 - 10 00:00 UTC);

        let bytes = zstd::encode_all(
            io::Cursor::new(concat!(
                r#"{"security_id": "S-1318698","#,
                r#""event_id": "E-3287361-DIV","#,
                r#""event": "DIV","#,
                r#""issuer_name": "VanEck ETF Trust","#,
                r#""security_type": "ETF","#,
                r#""primary_exchange": "USBATS","#,
                r#""exchange": "USBATS","#,
                r#""operating_mic": "BATS","#,
                r#""symbol": "HYD","#,
                r#""nasdaq_symbol": "HYD","#,
                r#""local_code": "HYD","#,
                r#""local_code_resulting": null,"#,
                r#""isin": "US92189H4092","#,
                r#""isin_resulting": null,"#,
                r#""us_code": "92189H409","#,
                r#""status": "A","#,
                r#""ex_date": "2024-05-01","#,
                r#""factor": 0.995833170541121,"#,
                r#""close": 51.19,"#,
                r#""currency": "USD","#,
                r#""sentiment": 0.998241844110178,"#,
                r#""reason": 17,"#,
                r#""gross_dividend": 0.2133,"#,
                r#""dividend_currency": "USD","#,
                r#""frequency": "MNT","#,
                r#""option": 1,"#,
                r#""detail": "INT Dividend (cash) of USD0.2133/ETF","#,
                r#""ts_created": "1970-01-01T00:00:00.000000000Z"}
"#,
            )),
            0,
        )
        .unwrap();

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!(
                "/v{API_VERSION}/adjustment_factors.get_range"
            )))
            .and(body_contains(
                "start",
                start.unix_timestamp_nanos().to_string(),
            ))
            .and(body_contains("stype_in", "raw_symbol"))
            .and(body_contains("symbols", "MSFT"))
            .and(body_contains("security_types", "EQS"))
            .respond_with(ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_bytes(bytes))
            .mount(&mock_server)
            .await;

        let mut client = client(&mock_server);
        let res = client
            .adjustment_factors()
            .get_range(
                &GetRangeParams::builder()
                    .start(start)
                    .security_types([SecurityType::Eqs])
                    .countries([Country::Us])
                    .symbols("MSFT")
                    .build(),
            )
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
        let res = &res[0];
        assert_eq!(res.event, Event::Div);
        assert_eq!(res.security_type, SecurityType::Etf);
        assert_eq!(res.status, AdjustmentStatus::Apply);
        assert_eq!(res.ex_date, date!(2024 - 05 - 01));
        assert_eq!(res.dividend_currency, Some(Currency::Usd));
        assert_eq!(res.frequency, Some(Frequency::Monthly));
    }
}
