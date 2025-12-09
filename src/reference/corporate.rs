//! The corporate actions API.

use std::{collections::HashMap, fmt::Display};

use dbn::{Compression, SType};
use serde::Deserialize;
use time::{Date, OffsetDateTime};
use typed_builder::TypedBuilder;

use crate::{
    deserialize::{deserialize_date_time, deserialize_opt_date_time_hash_map},
    historical::{handle_zstd_jsonl_response, AddToForm, ReqwestForm},
    reference::{
        Action, Country, Currency, End, Event, EventSubType, Fraction, GlobalStatus, ListingSource,
        ListingStatus, MandVolu, OutturnStyle, PaymentType, SecurityType, Start,
    },
    DateTimeLike, Symbols,
};

/// A client for the corporate actions group of Reference API endpoints.
#[derive(Debug)]
pub struct CorporateActionsClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl CorporateActionsClient<'_> {
    /// Requests a new corporate actions time series from Databento.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_range(
        &mut self,
        params: &GetRangeParams,
    ) -> crate::Result<Vec<CorporateAction>> {
        let form = vec![
            ("index", params.index.to_string()),
            ("stype_in", params.stype_in.to_string()),
            ("symbols", params.symbols.to_api_string()),
            ("compression", Compression::Zstd.to_string()),
        ]
        .add_to_form(&Start(params.start))
        .add_to_form(&End(params.end))
        .add_to_form(&params.events)
        .add_to_form(&params.countries)
        .add_to_form(&Exchanges(&params.exchanges))
        .add_to_form(&params.security_types);
        let resp = self
            .inner
            .post("corporate_actions.get_range")?
            .form(&form)
            .send()
            .await?;
        let mut corporate_actions: Vec<CorporateAction> = handle_zstd_jsonl_response(resp).await?;
        match params.index {
            Index::EventDate => corporate_actions.sort_by_key(|a| a.event_date),
            Index::ExDate => corporate_actions.sort_by_key(|a| a.ex_date),
            Index::TsRecord => corporate_actions.sort_by_key(|a| a.ts_record),
        };
        Ok(corporate_actions)
    }
}

struct Exchanges<'a>(&'a Vec<String>);

impl<'a> AddToForm<Exchanges<'a>> for ReqwestForm {
    fn add_to_form(mut self, Exchanges(exchanges): &Exchanges) -> Self {
        if !exchanges.is_empty() {
            self.push(("exchanges", exchanges.join(",")));
        }
        self
    }
}

/// Which field to use for filtering and sorting.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Index {
    /// [`event_date`][CorporateAction::event_date].
    #[default]
    EventDate,
    /// [`ex_date`][CorporateAction::ex_date].
    ExDate,
    /// [`ts_record`][CorporateAction::ts_record].
    TsRecord,
}

/// The parameters for [`CorporateActionsClient::get_range()`]. Use
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
    /// An optional list of event types to filter for. By default all events are included.
    #[builder(default, setter(into))]
    pub events: Vec<Event>,
    /// An optional list of country codes to filter for. By default all countries are
    /// included.
    #[builder(default, setter(into))]
    pub countries: Vec<Country>,
    /// An optional list of listing exchanges to filter for. By default all exchanges are
    /// included.
    #[builder(default, setter(into))]
    pub exchanges: Vec<String>,
    /// An optional list of security types to filter for. By default all security types
    /// are included.
    #[builder(default, setter(into))]
    pub security_types: Vec<SecurityType>,
}

/// A record in the corporate actions response.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct CorporateAction {
    /*
     * Identifiers
     */
    /// The timestamp (UTC) the record last changed.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_record: OffsetDateTime,
    /// Unique corporate actions record identifier. Can be used to deduplicate records
    /// for the same event.
    pub event_unique_id: String,
    /// Event identifier unique at the event level.
    pub event_id: String,
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
     * Event / dates
     */
    /// Record action status code.
    pub event_action: Action,
    /// Event type. The event_id where applicable links all payments rows of an event
    /// together to show all the payment options.
    pub event: Event,
    /// Event subtype. This is only used for a limited number of events where the event
    /// data supplied falls into distinct sub-groupings.
    pub event_subtype: Option<EventSubType>,
    /// The main calendar date name for the event.
    pub event_date_label: String,
    /// The primary date associated with the event, often marking when the event is
    /// scheduled to occur or take effect.
    pub event_date: Option<Date>,
    /// The date on which the event was officially created or recorded in the system.
    pub event_created_date: Date,
    /// The date on which the event becomes effective or is executed, signifying when
    /// the changes or actions are officially recognized.
    pub effective_date: Option<Date>,
    /// The ex-dividend date.
    pub ex_date: Option<Date>,
    /// The date on which a company reviews its records to determine the eligible
    /// shareholders entitled to receive dividends or participate in the event. The
    /// record date is one business day after the ex-date.
    pub record_date: Option<Date>,
    /// Record date ID. This ID links all events for the same security that share the
    /// same record date.
    pub record_date_id: Option<String>,
    /// Related event type.
    pub related_event: Option<Event>,
    /// Direct link to another event.
    pub related_event_id: Option<String>,

    /*
     * Listing
     */
    /// Global status code. Indicates the global listing activity status of a security.
    pub global_status: GlobalStatus,
    /// Listing status code. Indicates the listing activity status at market level.
    pub listing_status: ListingStatus,
    /// ndicates if the listing level data in the record.
    pub listing_source: ListingSource,
    /// The date when the security is officially listed on the exchange and becomes
    /// available for trading.
    pub listing_date: Option<Date>,
    /// The date when the security is officially removed from the exchange and ceases to
    /// be traded.
    pub delisting_date: Option<Date>,

    /*
     * Exchange/issuer
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

    /*
     * Country
     */
    /// Listing country.
    pub listing_country: Option<Country>,
    /// Register country.
    pub register_country: Option<Country>,
    /// Trading currency.
    pub trading_currency: Option<Currency>,
    /// `true` if there is currently more than one listing in the market.
    pub multi_currency: bool,
    /// Market Segment Name.
    pub segment_mic_name: Option<String>,
    /// Market Identifier Code (MIC) as an ISO 10383 string.
    pub segment_mic: Option<String>,

    /*
     * Event / financials
     */
    /// Indicates mandatory and/or voluntary event participation requirement.
    pub mand_volu_flag: MandVolu,
    /// The priority order number sequence where populated is necessary to correctly
    /// calculate resultant cash and stock outcomes.
    pub rd_priority: Option<u32>,
    /// Lot Size. Indicates the minimum number of shares that can be acquired in one
    /// transaction.
    pub lot_size: Option<u32>,
    /// Par value amount.
    pub par_value: Option<f64>,
    /// Par value currency.
    pub par_value_currency: Option<Currency>,
    /// The date on which the dividend or payment is made to eligible shareholders.
    pub payment_date: Option<Date>,
    /// Due bills redemption date.
    pub duebills_redemption_date: Option<Date>,
    /// The earliest date from which a specific corporate action or event is valid,
    /// active, or can be exercised.
    pub from_date: Option<Date>,
    /// The final date by which the specific corporate action or event is valid, active,
    /// or must be completed.
    pub to_date: Option<Date>,
    /// Registration date.
    pub registration_date: Option<Date>,
    /// The date on which the specific corporate action or event begins or becomes
    /// effective.
    pub start_date: Option<Date>,
    /// The final date by which the corporate action or event is valid, active, or must
    /// be completed.
    pub end_date: Option<Date>,
    /// The date when the corporate action or event opens for participation or becomes
    /// available to stakeholders.
    pub open_date: Option<Date>,
    /// The final date by which participation in the corporate action or event must
    /// be completed.
    pub close_date: Option<Date>,
    /// The date when the subscription period for the event begins.
    pub start_subscription_date: Option<Date>,
    /// The date when the subscription period for the event ends.
    pub end_subscription_date: Option<Date>,
    /// The deadline by which participants must choose or elect an option related to the event.
    pub option_election_date: Option<Date>,
    /// The date from which withdrawal rights become effective, allowing participants to
    /// retract their election or participation.
    pub withdrawal_rights_from_date: Option<Date>,
    /// The final date by which participants can exercise their withdrawal rights,
    /// retracting their election or participation.
    pub withdrawal_rights_to_date: Option<Date>,
    /// The date on which the event notification is issued or made public.
    pub notification_date: Option<Date>,
    /// The closing date of the company's financial year.
    pub financial_year_end_date: Option<Date>,
    /// The anticipated date when the event or related transaction is expected to be
    /// completed.
    pub exp_completion_date: Option<Date>,
    /// Payment type.
    pub payment_type: Option<PaymentType>,
    /// Option number of the event where applicable - options are ORs.
    pub option_id: Option<String>,
    /// Serial number of the event where applicable - serials are ANDs.
    pub serial_id: Option<String>,
    /// Flag indicating the benefit the shareholder would receive by default in case of
    /// several options being offered to them.
    pub default_option_flag: Option<bool>,
    /// Payment currency.
    pub rate_currency: Option<Currency>,
    /// Ratio denominator is the existing holding.
    pub ratio_old: Option<f64>,
    /// Ratio numerator is the new holding.
    pub ratio_new: Option<f64>,
    /// Describes how fractions are handled in settlement calculations.
    pub fraction: Option<Fraction>,
    /// Style of outturn security.
    pub outturn_style: Option<OutturnStyle>,
    /// Security asset type.
    pub outturn_security_type: Option<SecurityType>,
    /// Outturn security.
    pub outturn_security_id: Option<String>,
    /// Outturn ISIN.
    pub outturn_isin: Option<String>,
    /// Outturn CUSIP.
    pub outturn_us_code: Option<String>,
    /// Outturn local code.
    pub outturn_local_code: Option<String>,
    /// Outturn Bloomberg composite ID.
    pub outturn_bbg_comp_id: Option<String>,
    /// Outturn Bloomberg composite ticker.
    pub outturn_bbg_comp_ticker: Option<String>,
    /// Outturn FIGI - Bloomberg exchange level ID.
    pub outturn_figi: Option<String>,
    /// Outturn FIGI - Bloomberg exchange level ticker.
    pub outturn_figi_ticker: Option<String>,
    /// The quantity range within which the shareholder can offer from their total
    /// holding in the event.
    pub min_offer_qty: Option<u64>,
    /// The quantity range within which the shareholder can offer from their total
    /// holding in the event.
    pub max_offer_qty: Option<u64>,
    /// If the quantity held by shareholder is within this range then they qualify for
    /// taking part in the event.
    pub min_qualify_qty: Option<u64>,
    /// If the quantity held by shareholder is within this range then they qualify for
    /// taking part in the event.
    pub max_qualify_qty: Option<u64>,
    /// The total quantity the company will accept from all the shareholders tendering
    /// their shares in the event for the event to be binding on the offeror.
    pub min_accept_qty: Option<u64>,
    /// The total quantity the company will accept from all the shareholders tendering
    /// their shares in the event for the event to be binding on the offeror.
    pub max_accept_qty: Option<u64>,
    /// If the event is via a tender process it is the cut off price at which all the
    /// bids are accepted.
    pub tender_strike_price: Option<f64>,
    /// If the event is via a tender then there is a price step in which one can put in
    /// the bids.
    pub tender_price_step: Option<f64>,
    /// Option expiry time.
    pub option_expiry_time: Option<String>,
    /// Option expiry time zone.
    pub option_expiry_tz: Option<String>,
    /// Withdrawal rights flag.
    pub withdrawal_rights_flag: Option<bool>,
    /// Withdrawal rights expiry time .
    pub withdrawal_rights_expiry_time: Option<String>,
    /// Withdrawal rights expiry time zone.
    pub withdrawal_rights_expiry_tz: Option<String>,
    /// Expiry time.
    pub expiry_time: Option<String>,
    /// Expiry time zone.
    pub expiry_tz: Option<String>,
    /// Event-specific date information.
    #[serde(deserialize_with = "deserialize_opt_date_time_hash_map")]
    pub date_info: HashMap<String, Option<OffsetDateTime>>,
    /// Event-specific payment information.
    pub rate_info: HashMap<String, Option<f64>>,
    /// Additional event-specific information.
    pub event_info: HashMap<String, Option<String>>,
    /// The timestamp (UTC) the record was added by Databento.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub ts_created: OffsetDateTime,
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Index::EventDate => write!(f, "event_date"),
            Index::ExDate => write!(f, "ex_date"),
            Index::TsRecord => write!(f, "ts_record"),
        }
    }
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
                r#"{"ts_record": "2023-10-10T03:37:14Z","#,
                r#""event_unique_id": "U-40179751345-16556634","#,
                r#""event_id": "E-9751345-RSPLT","#,
                r#""listing_id": "L-16556634","#,
                r#""listing_group_id": "LG-6556634","#,
                r#""security_id": "S-4633970","#,
                r#""issuer_id": "I-175515","#,
                r#""event_action": "U","#,
                r#""event": "RSPLT","#,
                r#""event_subtype": "CONSD","#,
                r#""event_date_label": "ex_date","#,
                r#""event_date": "2023-10-10","#,
                r#""event_created_date": "1929-09-30","#,
                r#""effective_date": null,"#,
                r#""ex_date": "2023-10-10","#,
                r#""record_date": null,"#,
                r#""record_date_id": "D-9751345","#,
                r#""related_event": null,"#,
                r#""related_event_id": null,"#,
                r#""global_status": "A","#,
                r#""listing_status": "L","#,
                r#""listing_source": "M","#,
                r#""listing_date": "2015-10-29","#,
                r#""delisting_date": null,"#,
                r#""issuer_name": "Borqs Technologies Inc","#,
                r#""security_type": "EQS","#,
                r#""security_description": "Ordinary Shares","#,
                r#""primary_exchange": "USNASD","#,
                r#""exchange": "USNASD","#,
                r#""operating_mic": "XNAS","#,
                r#""symbol": "BRQS","#,
                r#""nasdaq_symbol": "BRQS","#,
                r#""local_code": "BRQS","#,
                r#""isin": "VGG1466B1452","#,
                r#""us_code": "G1466B145","#,
                r#""bbg_comp_id": "BBG00B9RG1J6","#,
                r#""bbg_comp_ticker": "BRQS US","#,
                r#""figi": "BBG00B9RG1W1","#,
                r#""figi_ticker": "BRQS UR","#,
                r#""listing_country": "US","#,
                r#""register_country": "VG","#,
                r#""trading_currency": "USD","#,
                r#""multi_currency": false,"#,
                r#""segment_mic_name": "Capital Market","#,
                r#""segment_mic": "XNCM","#,
                r#""mand_volu_flag": "M","#,
                r#""rd_priority": 1,"#,
                r#""lot_size": 100,"#,
                r#""par_value": null,"#,
                r#""par_value_currency": "USD","#,
                r#""payment_date": null,"#,
                r#""duebills_redemption_date": null,"#,
                r#""from_date": null,"#,
                r#""to_date": null,"#,
                r#""registration_date": null,"#,
                r#""start_date": null,"#,
                r#""end_date": null,"#,
                r#""open_date": null,"#,
                r#""close_date": null,"#,
                r#""start_subscription_date": null,"#,
                r#""end_subscription_date": null,"#,
                r#""option_election_date": null,"#,
                r#""withdrawal_rights_from_date": null,"#,
                r#""withdrawal_rights_to_date": null,"#,
                r#""notification_date": null,"#,
                r#""financial_year_end_date": null,"#,
                r#""exp_completion_date": null,"#,
                r#""payment_type": "S","#,
                r#""option_id": "1","#,
                r#""serial_id": "1","#,
                r#""default_option_flag": true,"#,
                r#""rate_currency": "USD","#,
                r#""ratio_old": 12.0,"#,
                r#""ratio_new": 1.0,"#,
                r#""fraction": "U","#,
                r#""outturn_style": "NEWO","#,
                r#""outturn_security_type": "EQS","#,
                r#""outturn_security_id": "S-4633970","#,
                r#""outturn_isin": "VGG1466B1452","#,
                r#""outturn_us_code": "G1466B145","#,
                r#""outturn_local_code": "BRQS","#,
                r#""outturn_bbg_comp_id": "BBG00B9RG1J6","#,
                r#""outturn_bbg_comp_ticker": "BRQS US","#,
                r#""outturn_figi": "BBG00B9RG1W1","#,
                r#""outturn_figi_ticker": "BRQS UR","#,
                r#""min_offer_qty": null,"#,
                r#""max_offer_qty": null,"#,
                r#""min_qualify_qty": null,"#,
                r#""max_qualify_qty": null,"#,
                r#""min_accept_qty": null,"#,
                r#""max_accept_qty": null,"#,
                r#""tender_strike_price": null,"#,
                r#""tender_price_step": null,"#,
                r#""option_expiry_time": null,"#,
                r#""option_expiry_tz": null,"#,
                r#""withdrawal_rights_flag": null,"#,
                r#""withdrawal_rights_expiry_time": null,"#,
                r#""withdrawal_rights_expiry_tz": null,"#,
                r#""expiry_time": null,"#,
                r#""expiry_tz": null,"#,
                r#""date_info": {},"#,
                r#""rate_info": {"par_value_old": null, "par_value_new": null},"#,
                r#""event_info": {},"#,
                r#""ts_created": "1970-01-01T00:00:00.000000000Z"}
"#,
            )),
            0,
        )
        .unwrap();

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/corporate_actions.get_range")))
            .and(body_contains("index", "ts_record"))
            .and(body_contains("stype_in", "raw_symbol"))
            .and(body_contains("symbols", "MSFT"))
            .and(body_contains("events", "SHOCH"))
            .and(body_contains("countries", "US%2CCA"))
            .and(body_contains(
                "start",
                start.unix_timestamp_nanos().to_string(),
            ))
            .respond_with(ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_bytes(bytes))
            .mount(&mock_server)
            .await;

        let mut client = client(&mock_server);
        let res = client
            .corporate_actions()
            .get_range(
                &GetRangeParams::builder()
                    .start(start)
                    .events([Event::Shoch])
                    .countries([Country::Us, Country::Ca])
                    .index(Index::TsRecord)
                    .symbols("MSFT")
                    .build(),
            )
            .await
            .unwrap();
        assert_eq!(res.len(), 1);
        let res = &res[0];
        assert_eq!(res.event, Event::Rsplt);
        assert_eq!(res.event_action, Action::Updated);
        assert_eq!(res.global_status, GlobalStatus::Active);
        assert_eq!(res.listing_date, Some(date!(2015 - 10 - 29)));
        assert_eq!(res.security_type, Some(SecurityType::Eqs));
        assert_eq!(res.register_country, Some(Country::Vg));
        assert!(res.rate_info.contains_key("par_value_old"));
        assert!(res.rate_info.contains_key("par_value_new"));
    }
}
