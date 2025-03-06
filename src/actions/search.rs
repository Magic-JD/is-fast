use crate::config::load::Config;
use crate::search::search_type::{Search, SearchEngine};
use crate::transform::page::PageExtractor;
use crate::tui::browser::Browser;
use once_cell::sync::Lazy;
use std::thread::sleep;
use std::time::Duration;

static SEARCH_ENGINE: Lazy<SearchEngine> = Lazy::new(Config::get_search_engine);

pub fn run(search_term: String) {
    let mut browser = Browser::new();
    let mut last_error = None;
    let links = std::iter::repeat_with(|| SEARCH_ENGINE.search(&search_term))
        .take(4)
        .filter_map(|result| match result {
            Ok(links) if !links.is_empty() => Some(links),
            Err(e) => {
                last_error = Some(e.to_string());
                sleep(Duration::from_secs(1)); // Wait before retrying
                None
            }
            _ => None, // Empty links, try again
        })
        .next(); // Take the first successful result

    match links {
        Some(links) => {
            browser.browse(links, PageExtractor::from_url(), true);
        }
        None => {
            browser.shutdown();
            if let Some(error) = last_error {
                eprintln!("{}", error);
            } else {
                eprintln!("No links were found, no error detected");
            }
        }
    }
}
