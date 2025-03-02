use crate::links::extract::from_html;
use crate::scrapers::scrape::scrape;
use crate::tui::browser::Browser;

pub fn run(search_term: String) {
    let browser = Browser::new();
    let links = scrape(&format!(
        "https://html.duckduckgo.com/html/?q={}",
        &search_term
    ))
    .map(|html| from_html(&html))
    .unwrap_or_else(|_| vec![]);
    browser.browse(links, true);
}
