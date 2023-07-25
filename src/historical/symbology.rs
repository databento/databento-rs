//! Historical symbology API.

use std::collections::HashMap;

use dbn::{enums::SType, MappingInterval};
use reqwest::RequestBuilder;
use serde::Deserialize;
use typed_builder::TypedBuilder;

use crate::Symbols;

use super::DateRange;

/// A client for the symbology group of Historical API endpoints.
pub struct SymbologyClient<'a> {
    pub(crate) inner: &'a mut super::Client,
}

impl SymbologyClient<'_> {
    /// Resolves a list of symbols from an input symbology type to an output one.
    ///
    /// For example, resolves a raw symbol to an instrument ID: `ESM2` → `3403`.
    ///
    /// # Errors
    /// This function returns an error when it fails to communicate with the Databento API
    /// or the API indicates there's an issue with the request.
    pub async fn resolve(&mut self, params: &ResolveParams) -> crate::Result<Resolution> {
        let mut form = vec![
            ("dataset", params.dataset.to_string()),
            ("stype_in", params.stype_in.to_string()),
            ("stype_out", params.stype_out.to_string()),
            ("symbols", params.symbols.to_api_string()),
        ];
        params.date_range.add_to_form(&mut form);
        Ok(self
            .post("resolve")?
            .form(&form)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?)
    }

    fn post(&mut self, slug: &str) -> crate::Result<RequestBuilder> {
        self.inner.post(&format!("symbology.{slug}"))
    }
}

/// The parameters for [`SymbologyClient::resolve()`]. Use [`ResolveParams::builder()`]
/// to get a builder type with all the preset defaults.
#[derive(Debug, Clone, TypedBuilder)]
pub struct ResolveParams {
    /// The dataset code.
    pub dataset: String,
    /// The symbols to resolve.
    pub symbols: Symbols,
    /// The symbology type of the input `symbols`. Defaults to
    /// [`RawSymbol`](dbn::enums::SType::RawSymbol).
    #[builder(default = SType::RawSymbol)]
    pub stype_in: SType,
    /// The symbology type of the output `symbols`. Defaults to
    /// [`InstrumentId`](dbn::enums::SType::InstrumentId).
    #[builder(default = SType::InstrumentId)]
    pub stype_out: SType,
    /// The date range of the resolution.
    pub date_range: DateRange,
}

/// A symbology resolution from one symbology type to another.
#[derive(Debug, Clone, Deserialize)]
pub struct Resolution {
    /// A mapping from input symbol to a list of resolved symbols in the output
    /// symbology.
    pub mappings: HashMap<String, Vec<MappingInterval>>,
    /// A list of symbols that were resolved for part, but not all of the date range
    /// from the request.
    pub partial: Vec<String>,
    /// A list of symbols that were not resolved.
    pub not_found: Vec<String>,
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;
    use serde_json::json;
    use time::macros::date;
    use wiremock::{
        matchers::{basic_auth, method, path},
        Mock, MockServer, ResponseTemplate,
    };

    use super::*;
    use crate::{
        body_contains,
        historical::{HistoricalGateway, API_VERSION},
        HistoricalClient,
    };

    const API_KEY: &str = "test-API";

    #[tokio::test]
    async fn test_resolve() {
        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(basic_auth(API_KEY, ""))
            .and(path(format!("/v{API_VERSION}/symbology.resolve")))
            .and(body_contains("dataset", "GLBX.MDP3"))
            .and(body_contains("symbols", "ES.c.0%2CES.d.0"))
            .and(body_contains("stype_in", "continuous"))
            // default
            .and(body_contains("stype_out", "instrument_id"))
            .and(body_contains("start_date", "2023-06-14"))
            .and(body_contains("end_date", "2023-06-17"))
            .respond_with(ResponseTemplate::new(StatusCode::OK).set_body_json(json!({
                "mappings": {
                    "ES.c.0": [
                        {
                            "d0": "2023-06-14",
                            "d1": "2023-06-15",
                            "s": "10245"
                        },
                        {
                            "d0": "2023-06-15",
                            "d1": "2023-06-16",
                            "s": "10248"
                        }
                    ]
                },
                "partial": [],
                "not_found": ["ES.d.0"]
            })))
            .mount(&mock_server)
            .await;
        let mut target = HistoricalClient::with_url(
            mock_server.uri(),
            API_KEY.to_owned(),
            HistoricalGateway::Bo1,
        )
        .unwrap();
        let res = target
            .symbology()
            .resolve(
                &ResolveParams::builder()
                    .dataset(dbn::datasets::GLBX_MDP3.to_owned())
                    .symbols(vec!["ES.c.0", "ES.d.0"].into())
                    .stype_in(SType::Continuous)
                    .date_range((date!(2023 - 06 - 14), date!(2023 - 06 - 17)).into())
                    .build(),
            )
            .await
            .unwrap();
        assert_eq!(
            *res.mappings.get("ES.c.0").unwrap(),
            vec![
                MappingInterval {
                    start_date: time::macros::date!(2023 - 06 - 14),
                    end_date: time::macros::date!(2023 - 06 - 15),
                    symbol: "10245".to_owned()
                },
                MappingInterval {
                    start_date: time::macros::date!(2023 - 06 - 15),
                    end_date: time::macros::date!(2023 - 06 - 16),
                    symbol: "10248".to_owned()
                },
            ]
        );
        assert!(res.partial.is_empty());
        assert_eq!(res.not_found, vec!["ES.d.0"]);
    }
}
