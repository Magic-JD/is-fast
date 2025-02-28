use crate::links::extract::from_html;
use crate::scrapers::scrape::scrape;
use crate::tui::render::{loading, show, shutdown_with_error};

pub fn run(search_term: String) {
    loading().unwrap_or_else(|err| shutdown_with_error(&err.to_string()));
    let links = &scrape(&format!(
        "https://html.duckduckgo.com/html/?q={}",
        &search_term
    ))
    .map(|html| from_html(&html))
    .unwrap_or_else(|err| shutdown_with_error(&err.to_string()));
    show(&links);
}

