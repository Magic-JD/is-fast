use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::search::link::Link;
use crate::search::search_type::{Search, SearchEngine};
use crate::transform::page::PageExtractor;
use crate::tui::browser::Browser;
use once_cell::sync::Lazy;
use std::thread::sleep;
use std::time::Duration;

static SEARCH_ENGINE: Lazy<&SearchEngine> = Lazy::new(Config::get_search_engine);

pub fn run(search_term: &str, piped: bool) {
    match piped {
        true => run_piped(search_term),
        false => run_tui(search_term),
    }
}

fn run_tui(search_term: &str) {
    let mut browser = Browser::new();
    let links_result = find_links(search_term);
    match links_result {
        Ok(links) => {
            browser.browse(&links, &PageExtractor::from_url(), true);
        }
        Err(error) => {
            browser.shutdown();
            eprintln!("{error}");
        }
    }
}

fn run_piped(search_term: &str) {
    let links_result = find_links(search_term);
    match links_result.as_deref() {
        Ok([link, ..]) => {
            println!("{}", PageExtractor::from_url().get_plain_text(link));
        }
        Err(error) => {
            eprintln!("{error}");
        }
        Ok(&[]) => eprintln!("No links found, no error detected."),
    }
}

fn find_links(search_term: &str) -> Result<Vec<Link>, IsError> {
    let mut last_error = None;
    std::iter::repeat_with(|| SEARCH_ENGINE.search(search_term))
        .take(4)
        .find_map(|result| match result {
            Ok(links) if !links.is_empty() => Some(links),
            Err(e) => {
                last_error = Some(e.to_string());
                sleep(Duration::from_secs(1)); // Wait before retrying
                None
            }
            _ => None, // Empty links, try again
        })
        .ok_or(IsError::Search(last_error.unwrap_or_else(|| {
            String::from("No links were found, no error detected")
        })))
}
