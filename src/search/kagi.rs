use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Scrape;
use crate::errors::error::IsError::Search as SearchError;
use crate::search::link::Link;
use crate::search::scrape::REQWEST_CLIENT;
use crate::search::search_type::Search;
use reqwest::blocking::Response;
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct KagiSearch;

const API_KEY: &str = "KAGI_API_KEY";

impl KagiSearch {
    fn extract_variables(&self) -> Result<String, IsError> {
        let api_key = std::env::var(API_KEY).map_err(|_| {
            SearchError(format!("Unable to get the environment variable {API_KEY}",))
        })?;

        Ok(api_key)
    }

    pub fn get_links(&self, query: &str) -> Result<Vec<Link>, IsError> {
        let api_key = self.extract_variables()?;
        let url = format!("https://kagi.com/api/v0/search?q={query}");

        let response: Result<Value, IsError> = REQWEST_CLIENT
            .get(&url)
            .header("Authorization", format!("Bot {api_key}"))
            .send()
            .and_then(|resp| resp.error_for_status())
            .and_then(|resp| resp.json::<Value>())
            .map_err(|e| Scrape(format!("Request failed for {url}: {e}")));
        let response = response?;
        let data = &response["data"].as_array().unwrap();
        let search_results: Vec<Link> = data
            .iter()
            .filter(|item| item["t"] == 0)
            .map(|result| {
                Link::new(
                    result["title"].to_string().trim_matches('"').to_string(),
                    result["url"].to_string().trim_matches('"').to_string(),
                    Config::get_selectors(&result["url"].to_string()),
                )
            })
            .collect();

        Ok(search_results)
    }
}

impl Search for KagiSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        self.get_links(query)
    }
}
