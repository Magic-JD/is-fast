use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use crate::errors::error::IsError::Search as SearchError;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::UREQ_AGENT;
use crate::search_engine::search_type::Search;
use serde_json::from_str;

#[derive(serde::Deserialize)]
struct SearchResult {
    data: Vec<SearchItem>,
}

#[derive(serde::Deserialize)]
struct SearchItem {
    t: i32,
    #[serde(default)]
    url: String,
}

#[derive(Debug, Clone)]
pub struct KagiSearch;

const API_KEY: &str = "IS_FAST_KAGI_API_KEY";

impl KagiSearch {
    fn extract_variables() -> Result<String, IsError> {
        let api_key = std::env::var(API_KEY).map_err(|_| {
            SearchError(format!("Unable to get the environment variable {API_KEY}",))
        })?;

        Ok(api_key)
    }

    fn search_result_to_links(search_result: &SearchResult) -> Vec<Link> {
        search_result
            .data
            .iter()
            .filter(|item| item.t == 0)
            .map(|item| Link::new(&item.url))
            .collect()
    }

    fn request_results(api_key: &str, query: &str) -> Result<String, IsError> {
        let url = format!("https://kagi.com/api/v0/search?q={query}");
        UREQ_AGENT
            .get(&url)
            .header("Authorization", &format!("Bot {api_key}"))
            .call()
            .map_err(|e| Scrape(format!("Request failed for {url}: {e}")))
            .and_then(|response| {
                if response.status().is_success() {
                    response
                        .into_body()
                        .read_to_string()
                        .map_err(|e| Scrape(format!("Failed to read response body for {url}: {e}")))
                } else {
                    Err(Scrape(format!(
                        "Request failed for {url}: HTTP Status {}",
                        response.status()
                    )))
                }
            })
    }

    fn get_links(api_key: &str, query: &str) -> Result<Vec<Link>, IsError> {
        Self::request_results(api_key, query)
            .and_then(|json| {
                from_str::<SearchResult>(&json).map_err(|e| SearchError(e.to_string()))
            })
            .map(|search_result| Self::search_result_to_links(&search_result))
            .map_err(|e| SearchError(e.to_string()))
    }
}

impl Search for KagiSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        Self::extract_variables().and_then(|api_key| Self::get_links(&api_key, query))
    }
}
