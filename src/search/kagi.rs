use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use crate::errors::error::IsError::Search as SearchError;
use crate::search::link::Link;
use crate::search::scrape::REQWEST_CLIENT;
use crate::search::search_type::Search;
use reqwest::blocking::Response;
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
    #[serde(default)]
    title: String,
}

#[derive(Debug, Clone)]
pub struct KagiSearch;

const API_KEY: &str = "IS_FAST_KAGI_API_KEY";

impl KagiSearch {
    fn extract_variables(&self) -> Result<String, IsError> {
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
            .map(|item| {
                Link::new(
                    item.title.clone(),
                    item.url.clone(),
                    Config::get_selectors(&item.url),
                )
            })
            .collect()
    }

    fn request_results(&self, api_key: &str, query: &str) -> Result<String, IsError> {
        let url = format!("https://kagi.com/api/v0/search?q={query}");
        REQWEST_CLIENT
            .get(&url)
            .header("Authorization", format!("Bot {api_key}"))
            .send()
            .and_then(Response::error_for_status)
            .and_then(Response::text)
            .map_err(|e| Scrape(format!("Request failed for {url}: {e}")))
    }

    fn get_links(&self, api_key: &str, query: &str) -> Result<Vec<Link>, IsError> {
        self.request_results(api_key, query)
            .and_then(|json| {
                from_str::<SearchResult>(&json).map_err(|e| SearchError(e.to_string()))
            })
            .map(|search_result| Self::search_result_to_links(&search_result))
            .map_err(|e| SearchError(e.to_string()))
    }
}

impl Search for KagiSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        self.extract_variables()
            .and_then(|api_key| self.get_links(&api_key, query))
    }
}
