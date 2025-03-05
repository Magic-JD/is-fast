use crate::transform::link::get_links;
use crate::transform::page::PageExtractor;
use crate::tui::browser::Browser;
use std::thread::sleep;
use std::time::Duration;

pub fn run(search_term: String) {
    let browser = Browser::new();
    let links = std::iter::repeat_with(|| get_links(&search_term))
        .take(4)
        .inspect(|links| {
            if links.is_empty() {
                // Wait to retry
                sleep(Duration::from_secs(1));
            }
        })
        .find(|links| !links.is_empty())
        .unwrap_or_default();
    browser.browse(links, PageExtractor::from_url(), true);
}
