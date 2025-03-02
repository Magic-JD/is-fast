use std::thread::sleep;
use std::time::Duration;
use crate::links::extract::from_html;
use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use crate::tui::browser::Browser;

pub fn run(search_term: String) {
    let browser = Browser::new();
    let mut links = get_links(&search_term);
    let mut retry_count = 3;
    while links.is_empty() && retry_count > 0 {
        //Intermittent ddg failure -> sleep and retry
        sleep(Duration::from_secs(2));
        retry_count -= 1;
        links = get_links(&search_term);
    }
    browser.browse(links, true);
}

fn get_links(search_term: &String) -> Vec<Link> {
    scrape(&format!(
        "https://html.duckduckgo.com/html/?q={}",
        &search_term
    ))
        .map(|html| from_html(&html))
        .unwrap_or_else(|_| vec![])
}
