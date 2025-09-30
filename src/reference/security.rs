//! The security master API.

use std::fmt::Display;

use dbn::{Compression, SType};
use reqwest::RequestBuilder;
use serde::Deserialize;
use time::{Date, OffsetDateTime};
use typed_builder::TypedBuilder;

use crate::{
    deserialize::deserialize_date_time,
    historical::{handle_zstd_jsonl_response, AddToForm},
    reference::{
        Country, Currency, DateTimeLike, End, ListingSource, ListingStatus, SecurityType, Start,
        Voting,
    },
    Symbols,
};

/// A client for the security master group of Reference API endpoints.
#[derive(Debug)]
pub struct SecurityMasterClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl SecurityMasterClient<'_> {
    /// Requests a new security master time series from Databento.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_range(
        &mut self,
        params: &GetRangeParams,
    ) -> crate::Result<Vec<SecurityMaster>> {
        let form = vec![
            ("index", params.index.to_string()),
            ("stype_in", params.stype_in.to_string()),
            ("symbols", params.symbols.to_api_string()),
            ("compression", Compression::Zstd.to_string()),
        ]
        .add_to_form(&Start(params.start))
        .add_to_form(&End(params.end))
        .add_to_form(&params.countries)
        .add_to_form(&params.security_types);
        let resp = self.post("get_range")?.form(&form).send().await?;
        let mut security_masters = handle_zstd_jsonl_response::<SecurityMaster>(resp).await?;
        match params.index {
            Index::TsEffective => security_masters.sort_by_key(|s| s.ts_effective),
            Index::TsRecord => security_masters.sort_by_key(|s| s.ts_record),
        };
        Ok(security_masters)
    }

    /// Requests the latest security master from Databento.
    ///
    /// The resulting data will be sorted by `ts_effective`.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_last(&mut self, params: &GetLastParams) -> crate::Result<Vec<SecurityMaster>> {
        let form = vec![
            ("stype_in", params.stype_in.to_string()),
            ("symbols", params.symbols.to_api_string()),
            ("compression", Compression::Zstd.to_string()),
        ]
        .add_to_form(&params.countries)
        .add_to_form(&params.security_types);
        let resp = self.post("get_last")?.form(&form).send().await?;
        let mut security_masters = handle_zstd_jsonl_response::<SecurityMaster>(resp).await?;
        security_masters.sort_by_key(|s| s.ts_effective);
        Ok(security_masters)
    }

    fn post(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.inner.post(&format!("security_master.{slug}"))
    }
}

/// Which field to use for filtering and sorting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Index {
    /// [`ts_effective`][SecurityMaster::ts_effective].
    #[default]
    TsEffective,
    /// [`ts_record`][SecurityMaster::ts_record].
    TsRecord,
}

/// The parameters for [`SecurityMasterClient::get_range()`]. Use
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
    /// The timestamp to use for filtering.
    #[builder(default)]
    pub index: Index,
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

/// The parameters for [`SecurityMasterClient::get_last()`]. Use
/// [`GetLastParams::builder()`] to get a builder type with all the preset defaults.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct GetLastParams {
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

/// A record in the security master response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SecurityMaster {
    /*
     * Identifiers
     */
    /// The timestamp (UTC) the record last changed.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_record: OffsetDateTime,
    /// The timestamp (UTC) the record details are effective from.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_effective: OffsetDateTime,
    /// Unique listing numerical ID. Concatenation of a sequence number and the
    /// `listing_group_id`.
    pub listing_id: String,
    /// Groups all listings for the same security on a specific exchange, often in
    /// different trading currencies.
    pub listing_group_id: String,
    /// Security level numerical ID. Can be used to link all multiple listings together.
    pub security_id: String,
    /// Issuer level numerical ID. Can be used to link all securities of a company
    /// together.
    pub issuer_id: String,

    /*
     * Listing
     */
    /// Listing status code. Indicates the listing activity status at market level.
    pub listing_status: ListingStatus,
    /// Indicates if the listing level data in the record is (M)ain or (S)econdary.
    pub listing_source: ListingSource,
    /// Listing creation date.
    pub listing_created_date: Date,
    /// Listing date.
    pub listing_date: Option<Date>,
    /// Delisting date.
    pub delisting_date: Option<Date>,

    /*
     * Exchange
     */
    /// The issuer name.
    pub issuer_name: String,
    /// The security type.
    pub security_type: Option<SecurityType>,
    /// The security description.
    pub security_description: String,
    /// Exchange code for the primary security.
    pub primary_exchange: Option<String>,
    /// Exchange code for the listing.
    pub exchange: String,
    /// Market Identifier Code (MIC) as an ISO 10383 string.
    pub operating_mic: Option<String>,

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
    /// ISIN global level identifier as an ISO 6166 string.
    pub isin: Option<String>,
    /// US domestic CUSIP.
    pub us_code: Option<String>,
    /// Bloomberg composite global ID.
    pub bbg_comp_id: Option<String>,
    /// Bloomberg composite ticker.
    pub bbg_comp_ticker: Option<String>,
    /// Bloomberg FIGI, that is exchange level ID.
    pub figi: Option<String>,
    /// Bloomberg exchange level ticker.
    pub figi_ticker: Option<String>,
    /// Financial Instrument Short Name.
    pub fisn: Option<String>,
    /// Legal Entity Identifier.
    pub lei: Option<String>,
    /// Standard Industrial Classification Code.
    pub sic: Option<String>,
    /// Central Index Key.
    pub cik: Option<String>,
    /// Global Industry Standard Classification.
    pub gics: Option<String>,
    /// North American Industrial Classification System.
    pub naics: Option<String>,
    /// Complementary Identification Code.
    pub cic: Option<String>,
    /// Classification of Financial Instruments as an ISO 10962 string.
    pub cfi: Option<String>,

    /*
     * Country
     */
    /// Country of incorporation code of the issuer.
    pub incorporation_country: Country,
    /// Listing country.
    pub listing_country: Option<Country>,
    /// Register country.
    pub register_country: Option<Country>,
    /// Trading currency.
    pub trading_currency: Option<Currency>,
    /// `true` if there is currently more than one listing in the market.
    pub multi_currency: bool,

    /*
     * Financials
     */
    /// Market Segment Name.
    pub segment_mic_name: Option<String>,
    /// Market Identifier Code (MIC) as an ISO 10383 string.
    pub segment_mic: Option<String>,
    /// Security structure.
    pub structure: Option<String>,
    /// Lot Size. Indicates the minimum number of shares that can be acquired in one
    /// transaction.
    pub lot_size: Option<u32>,
    /// Par value amount.
    pub par_value: Option<f64>,
    /// Par value currency.
    pub par_value_currency: Option<Currency>,
    /// Voting or non-voting rights.
    pub voting: Option<Voting>,
    /// Number of votes per security.
    pub vote_per_sec: Option<f64>,
    /// Shares outstanding.
    pub shares_outstanding: Option<u64>,
    /// Effective date for `shares_outstanding`.
    pub shares_outstanding_date: Option<Date>,

    /// The timestamp (UTC) the record was added by Databento.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_created: OffsetDateTime,
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TsEffective => f.write_str("ts_effective"),
            Self::TsRecord => f.write_str("ts_record"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use reqwest::StatusCode;
    use rstest::*;
    use time::macros::datetime;
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

    #[rstest]
    #[tokio::test]
    async fn test_endpoint(#[values("get_last", "get_range")] endpoint: &str) {
        let _ = tracing_subscriber::FmtSubscriber::builder()
            .with_test_writer()
            .try_init();
        let start = datetime!(2023- 10 - 10 00:00 UTC);

        let bytes = zstd::encode_all(
            io::Cursor::new(concat!(
                r#"{"ts_record": "2009-05-12T13:44:05Z","#,
                r#""ts_effective": "2000-07-04T00:00:00Z","#,
                r#""listing_id": "L-211","#,
                r#""listing_group_id": "LG-81068","#,
                r#""security_id": "S-516531","#,
                r#""issuer_id": "I-2112","#,
                r#""listing_status": "L","#,
                r#""listing_source": "M","#,
                r#""listing_created_date": "2001-01-06","#,
                r#""listing_date": "1996-09-30","#,
                r#""delisting_date": null,"#,
                r#""issuer_name": "Sun Life Financial Services of Canada Inc.","#,
                r#""security_type": null,"#,
                r#""security_description": "Ordinary Shares","#,
                r#""primary_exchange": "CATSE","#,
                r#""exchange": "USNYSE","#,
                r#""operating_mic": "XBEY","#,
                r#""symbol": "SLF","#,
                r#""nasdaq_symbol": "SLF","#,
                r#""local_code": "SOLA","#,
                r#""isin": "CA8667961053","#,
                r#""us_code": "866796105","#,
                r#""bbg_comp_id": "BBG000BRM1N5","#,
                r#""bbg_comp_ticker": "SLF LB","#,
                r#""figi": "BBG000BRM1Y3","#,
                r#""figi_ticker": "SLF LB","#,
                r#""fisn": null,"#,
                r#""lei": null,"#,
                r#""sic": "CDA","#,
                r#""cik": "Share Depository Certificate","#,
                r#""gics": null,"#,
                r#""naics": null,"#,
                r#""cic": "USD","#,
                r#""cfi": "I","#,
                r#""incorporation_country": "CA","#,
                r#""listing_country": "LB","#,
                r#""register_country": "LB","#,
                r#""trading_currency": "USD","#,
                r#""multi_currency": false,"#,
                r#""segment_mic_name": null,"#,
                r#""segment_mic": null,"#,
                r#""structure": null,"#,
                r#""lot_size": 1,"#,
                r#""par_value": null,"#,
                r#""par_value_currency": null,"#,
                r#""voting": "M","#,
                r#""vote_per_sec": null,"#,
                r#""shares_outstanding": 14920000,"#,
                r#""shares_outstanding_date": "2000-07-04","#,
                r#""ts_created": "1970-01-01T00:00:00.000000000Z"}
"#,
            )),
            0,
        )
        .unwrap();

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/security_master.{endpoint}")))
            .and(body_contains("stype_in", "raw_symbol"))
            .and(body_contains("symbols", "MSFT"))
            .and(body_contains("security_types", "EQS"))
            .respond_with(ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_bytes(bytes))
            .mount(&mock_server)
            .await;

        let mut client = client(&mock_server);
        let res = if endpoint == "get_last" {
            client
                .security_master()
                .get_last(
                    &GetLastParams::builder()
                        .security_types([SecurityType::Eqs])
                        .countries([Country::Us])
                        .symbols("MSFT")
                        .build(),
                )
                .await
        } else {
            client
                .security_master()
                .get_range(
                    &GetRangeParams::builder()
                        .start(start)
                        .security_types([SecurityType::Eqs])
                        .countries([Country::Us])
                        .symbols("MSFT")
                        .build(),
                )
                .await
        }
        .unwrap();
        assert_eq!(res.len(), 1);
        let res = &res[0];
        assert_eq!(res.listing_status, ListingStatus::Listed);
        assert_eq!(res.listing_source, ListingSource::Main);
        assert!(res.security_type.is_none());
        assert_eq!(res.incorporation_country, Country::Ca);
        assert_eq!(res.voting, Some(Voting::Multiple));
    }
}
