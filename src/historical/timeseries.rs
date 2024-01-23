//! The historical timeseries API.

use std::num::NonZeroU64;

use dbn::{Compression, Encoding, SType, Schema, VersionUpgradePolicy};
use futures::TryStreamExt;
use reqwest::{header::ACCEPT, RequestBuilder};
use tokio::io::AsyncReadExt;
use typed_builder::TypedBuilder;

use crate::Symbols;

use super::{check_http_error, DateTimeRange};

// Re-export because it's returned.
pub use dbn::decode::AsyncDbnDecoder;

/// A client for the timeseries group of Historical API endpoints.
pub struct TimeseriesClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl TimeseriesClient<'_> {
    /// Makes a streaming request for timeseries data from Databento.
    ///
    /// This method returns a stream decoder. For larger requests, consider using
    /// [`BatchClient::submit_job()`](super::batch::BatchClient::submit_job()).
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn get_range(
        &mut self,
        params: &GetRangeParams,
    ) -> crate::Result<AsyncDbnDecoder<impl AsyncReadExt>> {
        let mut form = vec![
            ("dataset", params.dataset.to_string()),
            ("schema", params.schema.to_string()),
            ("encoding", Encoding::Dbn.to_string()),
            ("compression", Compression::ZStd.to_string()),
            ("stype_in", params.stype_in.to_string()),
            ("stype_out", params.stype_out.to_string()),
            ("symbols", params.symbols.to_api_string()),
        ];
        params.date_time_range.add_to_form(&mut form);
        if let Some(limit) = params.limit {
            form.push(("limit", limit.to_string()));
        }
        let resp = self
            .post("get_range")?
            // unlike almost every other request, it's not JSON
            .header(ACCEPT, "application/octet-stream")
            .form(&form)
            .send()
            .await?;
        let stream = check_http_error(resp)
            .await?
            .error_for_status()?
            .bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
        let reader = tokio_util::io::StreamReader::new(stream);
        let mut decoder: AsyncDbnDecoder<_> = AsyncDbnDecoder::with_zstd_buffer(reader).await?;
        decoder.set_upgrade_policy(params.upgrade_policy);
        Ok(decoder)
    }

    fn post(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.inner.post(&format!("timeseries.{slug}"))
    }
}

/// The parameters for [`TimeseriesClient::get_range()`]. Use
/// [`GetRangeParams::builder()`] to get a builder type with all the preset defaults.
#[derive(Debug, Clone, TypedBuilder, PartialEq, Eq)]
pub struct GetRangeParams {
    /// The dataset code.
    #[builder(setter(transform = |dt: impl ToString| dt.to_string()))]
    pub dataset: String,
    /// The symbols to filter for.
    #[builder(setter(into))]
    pub symbols: Symbols,
    /// The data record schema.
    pub schema: Schema,
    /// The request time range.
    #[builder(setter(into))]
    pub date_time_range: DateTimeRange,
    /// The symbology type of the input `symbols`. Defaults to
    /// [`RawSymbol`](dbn::enums::SType::RawSymbol).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// The symbology type of the output `symbols`. Defaults to
    /// [`InstrumentId`](dbn::enums::SType::InstrumentId).
    #[builder(default = SType::InstrumentId)]
    pub stype_out: SType,
    /// The optional maximum number of records to return. Defaults to no limit.
    #[builder(default)]
    pub limit: Option<NonZeroU64>,
    /// How to decode DBN from prior versions. Defaults to upgrade.
    #[builder(default = VersionUpgradePolicy::Upgrade)]
    pub upgrade_policy: VersionUpgradePolicy,
}

#[cfg(test)]
mod tests {
    use dbn::record::TradeMsg;
    use reqwest::StatusCode;
    use time::macros::datetime;
    use wiremock::{
        matchers::{basic_auth, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{
        body_contains,
        historical::{HistoricalGateway, API_VERSION},
        zst_test_data_path, HistoricalClient,
    };

    const API_KEY: &str = "test-API";

    #[tokio::test]
    async fn test_get_range() {
        const START: time::OffsetDateTime = datetime!(2023 - 06 - 14 00:00 UTC);
        const END: time::OffsetDateTime = datetime!(2023 - 06 - 17 00:00 UTC);
        const SCHEMA: Schema = Schema::Trades;

        let mock_server = MockServer::start().await;
        let bytes = tokio::fs::read(zst_test_data_path(SCHEMA)).await.unwrap();
        Mock::given(method("POST"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/timeseries.get_range")))
            .and(body_contains("dataset", "XNAS.ITCH"))
            .and(body_contains("schema", "trades"))
            .and(body_contains("symbols", "SPOT%2CAAPL"))
            .and(body_contains(
                "start",
                START.unix_timestamp_nanos().to_string(),
            ))
            .and(body_contains("end", END.unix_timestamp_nanos().to_string()))
            // // default
            .and(body_contains("stype_in", "raw_symbol"))
            .and(body_contains("stype_out", "instrument_id"))
            .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_bytes(bytes))
            .mount(&mock_server)
            .await;
        let mut target = HistoricalClient::with_url(
            mock_server.uri(),
            API_KEY.to_owned(),
            HistoricalGateway::Bo1,
        )
        .unwrap();
        let mut decoder = target
            .timeseries()
            .get_range(
                &GetRangeParams::builder()
                    .dataset(dbn::datasets::XNAS_ITCH)
                    .schema(SCHEMA)
                    .symbols(vec!["SPOT", "AAPL"])
                    .date_time_range((START, END))
                    .build(),
            )
            .await
            .unwrap();
        assert_eq!(decoder.metadata().schema.unwrap(), SCHEMA);
        // Two records
        decoder.decode_record::<TradeMsg>().await.unwrap().unwrap();
        decoder.decode_record::<TradeMsg>().await.unwrap().unwrap();
        assert!(decoder.decode_record::<TradeMsg>().await.unwrap().is_none());
    }
}
