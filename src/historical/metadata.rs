use std::collections::HashMap;

use dbn::enums::{Encoding, Schema};
use serde::Deserialize;
use typed_builder::TypedBuilder;

use super::DateTimeRange;

pub struct MetadataClient<'a> {
    pub(crate) inner: &'a mut reqwest::Client,
}

impl MetadataClient<'_> {
    pub async fn list_publishers(&mut self) -> anyhow::Result<HashMap<String, u16>> {
        todo!()
    }

    pub async fn list_datasets(&mut self) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    pub async fn list_schemas(&mut self, dataset: &str) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    pub async fn list_fields(
        &mut self,
        params: &ListFieldsParams<'_>,
    ) -> anyhow::Result<HashMap<String, HashMap<Encoding, HashMap<Schema, Vec<String>>>>> {
        todo!()
    }

    pub async fn list_unit_prices(
        &mut self,
        params: &ListUnitPricesParams<'_>,
    ) -> anyhow::Result<HashMap<FeedMode, HashMap<Schema, f64>>> {
        todo!()
    }

    pub async fn get_dataset_condition(
        &mut self,
        params: &GetDatasetConditionParams<'_>,
    ) -> anyhow::Result<Vec<DatasetConditionDetail>> {
        todo!()
    }

    pub async fn get_dataset_range(&mut self, dataset: &str) -> anyhow::Result<DatasetRange> {
        Ok(self
            .inner
            .get("/v0/metadata.get_dataset_range")
            .query(&["dataset", dataset])
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn get_record_count(&mut self) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    pub async fn get_billable_size(&mut self) -> anyhow::Result<Vec<String>> {
        todo!()
    }

    pub async fn get_cost(&mut self) -> anyhow::Result<Vec<String>> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedMode {
    Historical,
    HistoricalStreaming,
    Live,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum DatasetCondition {
    Available,
    Degraded,
    Pending,
    Missing,
    #[deprecated(since = "0.1.0")]
    Bad,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ListFieldsParams<'a> {
    #[builder(default)]
    pub dataset: Option<&'a str>,
    #[builder(default)]
    pub encoding: Option<Encoding>,
    #[builder(default)]
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ListUnitPricesParams<'a> {
    pub dataset: &'a str,
    #[builder(default)]
    pub feed_mode: Option<FeedMode>,
    #[builder(default)]
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct GetDatasetConditionParams<'a> {
    pub dataset: &'a str,
    #[builder(default)]
    pub date_range: Option<DateTimeRange>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetConditionDetail {
    pub date: time::Date,
    pub condition: DatasetCondition,
    pub last_modified_date: time::Date,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatasetRange {
    pub start_date: time::Date,
    pub end_date: time::Date,
}
