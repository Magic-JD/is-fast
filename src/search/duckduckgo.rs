use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::search::link::Link;
use crate::search::scrape::scrape;
use crate::search::search_type::Search;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DuckDuckGoSearch;
impl DuckDuckGoSearch {
    pub fn get_links(&self, search_term: &str) -> Result<Vec<Link>, IsError> {
        scrape(&format!(
            "https://html.duckduckgo.com/html/?q={}",
            &search_term
        ))
        .map(|html| self.links_from_html(&html))
    }

    fn links_from_html(&self, html: &str) -> Vec<Link> {
        let document = Html::parse_document(html);
        let selector_title = Selector::parse("a.result__a").unwrap();
        let selector_url = Selector::parse("a.result__url").unwrap();

        document
            .select(&selector_title)
            .zip(document.select(&selector_url))
            .map(|(title, url)| {
                let url = url.text().collect::<Vec<_>>().join(" ").trim().to_owned();
                Link::new(
                    title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
                    url.clone(),
                    Config::get_selectors(&url),
                )
            })
            .collect()
    }
}
impl Search for DuckDuckGoSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        self.get_links(query)
    }
}
