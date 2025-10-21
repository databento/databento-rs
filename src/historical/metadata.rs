//! The historical metadata download API.

use std::{collections::HashMap, num::NonZeroU64, str::FromStr};

use dbn::{Encoding, SType, Schema};
use reqwest::RequestBuilder;
use serde::{Deserialize, Deserializer};
use typed_builder::TypedBuilder;

use crate::{
    deserialize::deserialize_date_time,
    historical::{AddToForm, Limit, ReqwestForm},
    Symbols,
};

use super::{handle_response, AddToQuery, DateRange, DateTimeRange};

/// A client for the metadata group of Historical API endpoints.
#[derive(Debug)]
pub struct MetadataClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl MetadataClient<'_> {
    /// Lists the details of all publishers.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API.
    pub async fn list_publishers(&mut self) -> crate::Result<Vec<PublisherDetail>> {
        let resp = self.get("list_publishers")?.send().await?;
        handle_response(resp).await
    }

    /// Lists all available dataset codes on Databento.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn list_datasets(
        &mut self,
        date_range: Option<DateRange>,
    ) -> crate::Result<Vec<String>> {
        let mut builder = self.get("list_datasets")?;
        if let Some(date_range) = date_range {
            builder = builder.add_to_query(&date_range);
        }
        let resp = builder.send().await?;
        handle_response(resp).await
    }

    /// Lists all available schemas for the given `dataset`.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn list_schemas(&mut self, dataset: &str) -> crate::Result<Vec<Schema>> {
        let resp = self
            .get("list_schemas")?
            .query(&[("dataset", dataset)])
            .send()
            .await?;
        handle_response(resp).await
    }

    /// Lists all fields for a schema and encoding.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn list_fields(
        &mut self,
        params: &ListFieldsParams,
    ) -> crate::Result<Vec<FieldDetail>> {
        let builder = self.get("list_fields")?.query(&[
            ("encoding", params.encoding.as_str()),
            ("schema", params.schema.as_str()),
        ]);
        let resp = builder.send().await?;
        handle_response(resp).await
    }

    /// Lists unit prices for each data schema and feed mode in US dollars per gigabyte.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn list_unit_prices(
        &mut self,
        dataset: &str,
    ) -> crate::Result<Vec<UnitPricesForMode>> {
        let builder = self
            .get("list_unit_prices")?
            .query(&[("dataset", &dataset)]);
        let resp = builder.send().await?;
        handle_response(resp).await
    }

    /// Gets the dataset condition from Databento.
    ///
    /// Use this method to discover data availability and quality.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_dataset_condition(
        &mut self,
        params: &GetDatasetConditionParams,
    ) -> crate::Result<Vec<DatasetConditionDetail>> {
        let mut builder = self
            .get("get_dataset_condition")?
            .query(&[("dataset", &params.dataset)]);
        if let Some(ref date_range) = params.date_range {
            builder = builder.add_to_query(date_range);
        }
        let resp = builder.send().await?;
        handle_response(resp).await
    }

    /// Gets the available range for the dataset given the user's entitlements.
    ///
    /// Use this method to discover data availability.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_dataset_range(&mut self, dataset: &str) -> crate::Result<DatasetRange> {
        let resp = self
            .get("get_dataset_range")?
            .query(&[("dataset", dataset)])
            .send()
            .await?;
        handle_response(resp).await
    }

    /// Gets the record count of the time series data query.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_record_count(&mut self, params: &GetRecordCountParams) -> crate::Result<u64> {
        let form = ReqwestForm::new().add_to_form(params);
        let resp = self.post("get_record_count")?.form(&form).send().await?;
        handle_response(resp).await
    }

    /// Gets the billable uncompressed raw binary size for historical streaming or
    /// batched files.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_billable_size(
        &mut self,
        params: &GetBillableSizeParams,
    ) -> crate::Result<u64> {
        let form = ReqwestForm::new().add_to_form(params);
        let resp = self.post("get_billable_size")?.form(&form).send().await?;
        handle_response(resp).await
    }

    /// Gets the cost in US dollars for a historical streaming or batch download
    /// request. This cost respects any discounts provided by flat rate plans.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_cost(&mut self, params: &GetCostParams) -> crate::Result<f64> {
        let form = ReqwestForm::new().add_to_form(params);
        let resp = self.post("get_cost")?.form(&form).send().await?;
        handle_response(resp).await
    }

    fn get(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.inner.get(&format!("metadata.{slug}"))
    }

    fn post(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.inner.post(&format!("metadata.{slug}"))
    }
}

/// A type of data feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FeedMode {
    /// The historical batch data feed.
    Historical,
    /// The historical streaming data feed.
    HistoricalStreaming,
    /// The Live data feed for real-time and intraday historical.
    Live,
}

/// The condition of a dataset on a day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DatasetCondition {
    /// The data is available with no known issues.
    Available,
    /// The data is available, but there may be missing data or other correctness
    /// issues.
    Degraded,
    /// The data is not yet available, but may be available soon.
    Pending,
    /// The data is not available.
    Missing,
}

/// The details about a publisher.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct PublisherDetail {
    /// The publisher ID assigned by Databento, which denotes the dataset and venue.
    pub publisher_id: u16,
    /// The dataset code for the publisher.
    pub dataset: String,
    /// The venue for the publisher.
    pub venue: String,
    /// The publisher description.
    pub description: String,
}

/// The parameters for [`MetadataClient::list_fields()`]. Use
/// [`ListFieldsParams::builder()`] to get a builder type with all the preset defaults.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct ListFieldsParams {
    /// The encoding to request fields for.
    pub encoding: Encoding,
    /// The data record schema to request fields for.
    pub schema: Schema,
}

/// The details about a field in a schema.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct FieldDetail {
    /// The field name.
    pub name: String,
    /// The field type name.
    #[serde(rename = "type")]
    pub type_name: String,
}

/// The unit prices for a particular [`FeedMode`].
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct UnitPricesForMode {
    /// The data feed mode.
    pub mode: FeedMode,
    /// The unit prices in US dollars by data record schema.
    pub unit_prices: HashMap<Schema, f64>,
}

/// The parameters for [`MetadataClient::get_dataset_condition()`]. Use
/// [`GetDatasetConditionParams::builder()`] to get a builder type with all the preset
/// defaults.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct GetDatasetConditionParams {
    /// The dataset code.
    #[builder(setter(transform = |dataset: impl ToString| dataset.to_string()))]
    pub dataset: String,
    /// The UTC date request range with an inclusive start date and an inclusive end date.
    /// If `None` then will return all available dates.
    #[builder(default, setter(transform = |dr: impl Into<DateRange>| Some(dr.into())))]
    pub date_range: Option<DateRange>,
}

/// The condition of a dataset on a particular day.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct DatasetConditionDetail {
    /// The day of the described data.
    #[serde(deserialize_with = "deserialize_date")]
    pub date: time::Date,
    /// The condition code describing the quality and availability of the data on the
    /// given day.
    pub condition: DatasetCondition,
    /// The date when any schema in the dataset on the given day was last generated or
    /// modified. Will be None when `condition` is `Missing`.
    #[serde(deserialize_with = "deserialize_opt_date")]
    pub last_modified_date: Option<time::Date>,
}

/// The available range for a dataset.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct DatasetRange {
    /// The inclusive UTC start timestamp of the available range.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub start: time::OffsetDateTime,
    /// The exclusive UTC end timestamp of the available range.
    #[serde(deserialize_with = "deserialize_date_time")]
    pub end: time::OffsetDateTime,
    /// The available ranges for each available schema in the dataset.
    #[serde(rename = "schema")]
    pub range_by_schema: HashMap<Schema, DateTimeRange>,
}

impl From<DatasetRange> for DateTimeRange {
    fn from(DatasetRange { start, end, .. }: DatasetRange) -> Self {
        Self { start, end }
    }
}

/// The parameters for several metadata requests.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct GetQueryParams {
    /// The dataset code.
    #[builder(setter(transform = |dataset: impl ToString| dataset.to_string()))]
    pub dataset: String,
    /// The symbols to filter for.
    #[builder(setter(into))]
    pub symbols: Symbols,
    /// The data record schema.
    pub schema: Schema,
    /// The request range with an inclusive start and an exclusive end.
    #[builder(setter(into))]
    pub date_time_range: DateTimeRange,
    /// The symbology type of the input `symbols`. Defaults to
    /// [`RawSymbol`](dbn::enums::SType::RawSymbol).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// The optional maximum number of records to return. Defaults to no limit.
    #[builder(default)]
    pub limit: Option<NonZeroU64>,
}

/// The parameters for [`MetadataClient::get_record_count()`]. Use
/// [`GetRecordCountParams::builder()`] to get a builder type with all the preset
/// defaults.
pub type GetRecordCountParams = GetQueryParams;
/// The parameters for [`MetadataClient::get_billable_size()`]. Use
/// [`GetBillableSizeParams::builder()`] to get a builder type with all the preset
/// defaults.
pub type GetBillableSizeParams = GetQueryParams;
/// The parameters for [`MetadataClient::get_cost()`]. Use
/// [`GetCostParams::builder()`] to get a builder type with all the preset
/// defaults.
pub type GetCostParams = GetQueryParams;

impl AsRef<str> for FeedMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl FeedMode {
    /// Converts the enum to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            FeedMode::Historical => "historical",
            FeedMode::HistoricalStreaming => "historical-streaming",
            FeedMode::Live => "live",
        }
    }
}

impl FromStr for FeedMode {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "historical" => Ok(Self::Historical),
            "historical-streaming" => Ok(Self::HistoricalStreaming),
            "live" => Ok(Self::Live),
            _ => Err(crate::Error::internal(format_args!(
                "Unable to convert {s} to FeedMode"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for FeedMode {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(serde::de::Error::custom)
    }
}

impl AsRef<str> for DatasetCondition {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl DatasetCondition {
    /// Converts the enum to its `str` representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            DatasetCondition::Available => "available",
            DatasetCondition::Degraded => "degraded",
            DatasetCondition::Pending => "pending",
            DatasetCondition::Missing => "missing",
        }
    }
}

impl FromStr for DatasetCondition {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "available" => Ok(DatasetCondition::Available),
            "degraded" => Ok(DatasetCondition::Degraded),
            "pending" => Ok(DatasetCondition::Pending),
            "missing" => Ok(DatasetCondition::Missing),
            _ => Err(crate::Error::internal(format_args!(
                "Unable to convert {s} to DatasetCondition"
            ))),
        }
    }
}

impl<'de> Deserialize<'de> for DatasetCondition {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let str = String::deserialize(deserializer)?;
        FromStr::from_str(&str).map_err(serde::de::Error::custom)
    }
}

fn deserialize_date<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<time::Date, D::Error> {
    let date_str = String::deserialize(deserializer)?;
    time::Date::parse(&date_str, super::DATE_FORMAT).map_err(serde::de::Error::custom)
}

fn deserialize_opt_date<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<time::Date>, D::Error> {
    let opt_date_str: Option<String> = Option::deserialize(deserializer)?;
    match opt_date_str {
        Some(date_str) => {
            let date = time::Date::parse(&date_str, super::DATE_FORMAT)
                .map_err(serde::de::Error::custom)?;
            Ok(Some(date))
        }
        None => Ok(None),
    }
}

impl AddToForm<GetQueryParams> for ReqwestForm {
    fn add_to_form(mut self, param: &GetQueryParams) -> Self {
        self.push(("dataset", param.dataset.to_string()));
        self.push(("schema", param.schema.to_string()));
        self.push(("stype_in", param.stype_in.to_string()));
        self.push(("symbols", param.symbols.to_api_string()));
        self.add_to_form(&param.date_time_range)
            .add_to_form(&Limit(param.limit))
    }
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use serde_json::json;
    use time::macros::{date, datetime};
    use wiremock::{
        matchers::{basic_auth, method, path, query_param},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{
        historical::test_infra::{client, API_KEY},
        historical::API_VERSION,
    };

    #[tokio::test]
    async fn test_list_fields() {
        const ENC: Encoding = Encoding::Csv;
        const SCHEMA: Schema = Schema::Ohlcv1S;
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/metadata.list_fields")))
            .and(query_param("encoding", ENC.as_str()))
            .and(query_param("schema", SCHEMA.as_str()))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_json(json!([
                    {"name":"ts_event", "type": "uint64_t"},
                    {"name":"rtype", "type": "uint8_t"},
                    {"name":"open", "type": "int64_t"},
                    {"name":"high", "type": "int64_t"},
                    {"name":"low", "type": "int64_t"},
                    {"name":"close", "type": "int64_t"},
                    {"name":"volume", "type": "uint64_t"},
                ])),
            )
            .mount(&mock_server)
            .await;
        let mut target = client(&mock_server);
        let fields = target
            .metadata()
            .list_fields(
                &ListFieldsParams::builder()
                    .encoding(ENC)
                    .schema(SCHEMA)
                    .build(),
            )
            .await
            .unwrap();
        let exp = vec![
            FieldDetail {
                name: "ts_event".to_owned(),
                type_name: "uint64_t".to_owned(),
            },
            FieldDetail {
                name: "rtype".to_owned(),
                type_name: "uint8_t".to_owned(),
            },
            FieldDetail {
                name: "open".to_owned(),
                type_name: "int64_t".to_owned(),
            },
            FieldDetail {
                name: "high".to_owned(),
                type_name: "int64_t".to_owned(),
            },
            FieldDetail {
                name: "low".to_owned(),
                type_name: "int64_t".to_owned(),
            },
            FieldDetail {
                name: "close".to_owned(),
                type_name: "int64_t".to_owned(),
            },
            FieldDetail {
                name: "volume".to_owned(),
                type_name: "uint64_t".to_owned(),
            },
        ];
        assert_eq!(*fields, exp);
    }

    #[tokio::test]
    async fn test_list_unit_prices() {
        const SCHEMA: Schema = Schema::Tbbo;
        const DATASET: &str = "GLBX.MDP3";
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/metadata.list_unit_prices")))
            .and(query_param("dataset", DATASET))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_json(json!([
                    {
                        "mode": "historical",
                        "unit_prices": {
                            SCHEMA.as_str(): 17.89
                        }
                    },
                    {
                        "mode": "live",
                        "unit_prices": {
                            SCHEMA.as_str(): 34.22
                        }
                    }
                ])),
            )
            .mount(&mock_server)
            .await;
        let mut target = client(&mock_server);
        let prices = target.metadata().list_unit_prices(DATASET).await.unwrap();
        assert_eq!(
            prices,
            vec![
                UnitPricesForMode {
                    mode: FeedMode::Historical,
                    unit_prices: HashMap::from([(SCHEMA, 17.89)])
                },
                UnitPricesForMode {
                    mode: FeedMode::Live,
                    unit_prices: HashMap::from([(SCHEMA, 34.22)])
                }
            ]
        );
    }

    #[tokio::test]
    async fn test_get_dataset_condition() {
        const DATASET: &str = "GLBX.MDP3";
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!(
                "/v{API_VERSION}/metadata.get_dataset_condition"
            )))
            .and(query_param("dataset", DATASET))
            .and(query_param("start_date", "2022-05-17"))
            .and(query_param("end_date", "2022-05-19"))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_json(json!([
                    {
                        "date": "2022-05-17",
                        "condition": "available",
                        "last_modified_date": "2023-07-11",
                    },
                    {
                        "date": "2022-05-18",
                        "condition": "degraded",
                        "last_modified_date": "2022-05-19",
                    },
                    {
                        "date": "2022-05-19",
                        "condition": "missing",
                        "last_modified_date": null,
                    },
                ])),
            )
            .mount(&mock_server)
            .await;
        let mut target = client(&mock_server);
        let condition = target
            .metadata()
            .get_dataset_condition(
                &GetDatasetConditionParams::builder()
                    .dataset(DATASET.to_owned())
                    .date_range((date!(2022 - 05 - 17), time::Duration::days(2)))
                    .build(),
            )
            .await
            .unwrap();
        assert_eq!(condition.len(), 3);
        assert_eq!(
            condition[0],
            DatasetConditionDetail {
                date: date!(2022 - 05 - 17),
                condition: DatasetCondition::Available,
                last_modified_date: Some(date!(2023 - 07 - 11))
            }
        );
        assert_eq!(
            condition[1],
            DatasetConditionDetail {
                date: date!(2022 - 05 - 18),
                condition: DatasetCondition::Degraded,
                last_modified_date: Some(date!(2022 - 05 - 19))
            }
        );
        assert_eq!(
            condition[2],
            DatasetConditionDetail {
                date: date!(2022 - 05 - 19),
                condition: DatasetCondition::Missing,
                last_modified_date: None
            }
        );
    }

    #[tokio::test]
    async fn test_get_dataset_range() {
        const DATASET: &str = "XNAS.ITCH";
        let mock_server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/metadata.get_dataset_range")))
            .and(query_param("dataset", DATASET))
            .respond_with(
                ResponseTemplate::new(StatusCode::OK.as_u16()).set_body_json(json!({
                    "start": "2019-07-07T00:00:00.000000000Z",
                    "end": "2023-07-20T00:00:00.000000000Z",
                    "schema": {
                       "bbo-1m": {
                            "start": "2020-08-02T00:00:00.000000000Z",
                            "end": "2023-03-23T00:00:00.000000000Z"
                        },
                        "ohlcv-1s": {
                            "start": "2020-08-02T00:00:00.000000000Z",
                            "end": "2023-03-23T00:00:00.000000000Z"
                        },
                        "ohlcv-1m": {
                            "start": "2020-08-02T00:00:00.000000000Z",
                            "end": "2023-03-23T00:00:00.000000000Z"
                        },
                    }
                })),
            )
            .mount(&mock_server)
            .await;
        let mut target = client(&mock_server);
        let range = target.metadata().get_dataset_range(DATASET).await.unwrap();
        assert_eq!(range.start, datetime!(2019 - 07 - 07 00:00:00+00:00));
        assert_eq!(range.end, datetime!(2023 - 07 - 20 00:00:00.000000+00:00));
    }
}
