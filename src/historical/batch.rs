use std::{fmt::Write, num::NonZeroU64};

use dbn::enums::{Compression, Encoding, SType, Schema};
use serde::Deserialize;
use time::OffsetDateTime;
use typed_builder::TypedBuilder;

use crate::Symbols;

use super::{AddToQuery, DateTimeRange};

pub struct BatchClient<'a> {
    pub(crate) inner: &'a mut reqwest::Client,
}

impl BatchClient<'_> {
    pub async fn submit_job(&mut self, params: &SubmitJobParams<'_>) -> anyhow::Result<BatchJob> {
        let mut builder = self
            .inner
            .get("/v0/batch.submit_job")
            .query(&[
                ("dataset", params.dataset),
                ("schema", params.schema.as_str()),
                ("compression", params.compression.as_str()),
                ("encoding", Encoding::Dbn.as_str()),
                ("split_duration", params.split_duration.as_str()),
                ("packaging", params.packaging.as_str()),
                ("delivery", params.delivery.as_str()),
                ("stype_in", params.stype_in.as_str()),
                ("stype_out", params.stype_out.as_str()),
            ])
            .add_to_query(&params.date_time_range)
            .add_to_query(&params.symbols);
        if let Some(split_size) = params.split_size {
            builder = builder.query(&[("split_size", &split_size.to_string())])
        }
        if let Some(limit) = params.limit {
            builder = builder.query(&[("limit", &limit.to_string())])
        }
        Ok(builder.send().await?.json().await?)
    }

    pub async fn list_jobs(&mut self, params: &ListJobsParams) -> anyhow::Result<Vec<BatchJob>> {
        let mut builder = self.inner.get("/v0/batch.list_jobs");
        if let Some(ref states) = params.states {
            let states_str = states.iter().fold(String::new(), |mut acc, s| {
                if acc.is_empty() {
                    s.as_str().to_owned()
                } else {
                    write!(acc, ",{}", s.as_str()).unwrap();
                    acc
                }
            });
            builder = builder.query(&[("states", states_str)]);
        }
        if let Some(ref since) = params.since {
            builder = builder.query(&["since", &since.unix_timestamp_nanos().to_string()]);
        }
        Ok(builder.send().await?.json().await?)
    }

    pub async fn list_files(&mut self, job_id: &str) -> anyhow::Result<Vec<String>> {
        Ok(self
            .inner
            .get("/v0/batch.list_files")
            .query(&[("job_id", job_id)])
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn download() -> anyhow::Result<()> {
        todo!()
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum SplitDuration {
    #[default]
    Day,
    Week,
    Month,
    None,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum Packaging {
    #[default]
    None,
    Zip,
    Tar,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
pub enum Delivery {
    #[default]
    Download,
    S3,
    Disk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum JobState {
    Received,
    Queued,
    Processing,
    Done,
    Expired,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct SubmitJobParams<'a> {
    pub dataset: &'a str,
    pub symbols: Symbols<'a>,
    pub schema: Schema,
    pub date_time_range: DateTimeRange,
    #[builder(default = Compression::ZStd)]
    pub compression: Compression,
    #[builder(default)]
    pub split_duration: SplitDuration,
    #[builder(default)]
    pub split_size: Option<NonZeroU64>,
    #[builder(default)]
    pub packaging: Packaging,
    #[builder(default)]
    pub delivery: Delivery,
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    #[builder(default = SType::InstrumentId)]
    pub stype_out: SType,
    #[builder(default)]
    pub limit: Option<NonZeroU64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchJob {
    pub id: String,
    pub user_id: String,
    pub bill_id: String,
    pub cost_usd: f64,
    pub dataset: String,
    pub symbols: Vec<String>,
    pub stype_in: SType,
    pub stype_out: SType,
    pub schema: Schema,
    pub start: OffsetDateTime,
    pub end: OffsetDateTime,
    pub limit: Option<NonZeroU64>,
    pub encoding: Encoding,
    pub compression: Compression,
    pub split_duration: SplitDuration,
    pub split_size: Option<NonZeroU64>,
    pub split_symbols: bool,
    pub packaging: Packaging,
    pub delivery: Delivery,
    pub record_count: u64,
    pub billed_size: u64,
    pub actual_size: u64,
    pub package_size: u64,
    pub state: JobState,
    pub ts_received: OffsetDateTime,
    pub ts_queued: Option<OffsetDateTime>,
    pub ts_process_start: Option<OffsetDateTime>,
    pub ts_process_done: Option<OffsetDateTime>,
    pub ts_expiration: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, TypedBuilder)]
pub struct ListJobsParams {
    #[builder(default)]
    pub states: Option<Vec<JobState>>,
    #[builder(default)]
    pub since: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchFileDesc {
    pub filename: String,
    pub size: u64,
    pub hash: String,
    pub https_url: String,
    pub ftp_url: String,
}

impl SplitDuration {
    pub const fn as_str(&self) -> &'static str {
        match self {
            SplitDuration::Day => "day",
            SplitDuration::Week => "week",
            SplitDuration::Month => "month",
            SplitDuration::None => "none",
        }
    }
}

impl Packaging {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Packaging::None => "none",
            Packaging::Zip => "zip",
            Packaging::Tar => "tar",
        }
    }
}
impl Delivery {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Delivery::Download => "download",
            Delivery::S3 => "s3",
            Delivery::Disk => "disk",
        }
    }
}

impl JobState {
    pub const fn as_str(&self) -> &'static str {
        match self {
            JobState::Received => "received",
            JobState::Queued => "queued",
            JobState::Processing => "processing",
            JobState::Done => "done",
            JobState::Expired => "expired",
        }
    }
}
