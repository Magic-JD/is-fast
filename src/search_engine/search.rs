use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::search_engine::link::Link;
use crate::search_engine::search_type::{Search, SearchEngine};
use once_cell::sync::Lazy;
use std::thread::sleep;
use std::time::Duration;

static SEARCH_ENGINE: Lazy<&SearchEngine> = Lazy::new(Config::get_search_engine);

pub fn find_links(search_term: &str) -> Result<Vec<Link>, IsError> {
    let mut last_error = None;
    std::iter::repeat_with(|| SEARCH_ENGINE.search(search_term))
        .take(4)
        .find_map(|result| match result {
            Ok(links) if !links.is_empty() => Some(links),
            Err(e) => {
                log::debug!("failed to search links: {:?}", e);
                last_error = Some(e.to_string());
                sleep(Duration::from_secs(1)); // Wait before retrying
                None
            }
            _ => {
                log::debug!("failed to search links: no links found");
                sleep(Duration::from_secs(1)); // Wait before retrying
                None
            } // Empty links, try again
        })
        .ok_or(IsError::Search(last_error.unwrap_or_else(|| {
            String::from("No links were found, no error detected")
        })))
}
