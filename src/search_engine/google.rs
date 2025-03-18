use crate::errors::error::IsError;
use crate::errors::error::IsError::Search as SearchError;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::scrape;
use crate::search_engine::search_type::Search;
use serde_json::from_str;

#[derive(serde::Deserialize)]
struct SearchResult {
    items: Vec<SearchItem>,
}

#[derive(serde::Deserialize)]
struct SearchItem {
    link: String,
}

#[derive(Debug, Clone)]
pub struct GoogleSearch;

const API_KEY: &str = "IS_FAST_GOOGLE_API_KEY";
const SEARCH_ENGINE_ID: &str = "IS_FAST_GOOGLE_SEARCH_ENGINE_ID";

impl GoogleSearch {
    fn extract_variables() -> Result<(String, String), IsError> {
        let api_key = std::env::var(API_KEY).map_err(|_| {
            SearchError(format!("Unable to get the environment variable {API_KEY}",))
        })?;

        let search_engine_id = std::env::var(SEARCH_ENGINE_ID).map_err(|_| {
            SearchError(format!(
                "Unable to get the environment variable {SEARCH_ENGINE_ID}",
            ))
        })?;
        Ok((api_key, search_engine_id))
    }

    fn extract_links(
        api_key: &str,
        search_engine_id: &str,
        query: &str,
    ) -> Result<Vec<Link>, IsError> {
        scrape(&format!(
            "https://www.googleapis.com/customsearch/v1?key={api_key}&cx={search_engine_id}&q={query}",
        ))
        .and_then(|json| from_str::<SearchResult>(&json).map_err(|e| SearchError(e.to_string())))
        .map(|search_result| Self::search_result_to_links(&search_result))
        .map_err(|e| SearchError(e.to_string()))
    }

    fn search_result_to_links(search_result: &SearchResult) -> Vec<Link> {
        search_result
            .items
            .iter()
            .map(|item| Link::new(&item.link))
            .collect()
    }
}
impl Search for GoogleSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        Self::extract_variables().and_then(|(api_key, search_engine_id)| {
            Self::extract_links(&api_key, &search_engine_id, query)
        })
    }
}
