use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::Selector as SelectorError;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::scrape;
use crate::search_engine::search_type::Search;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DuckDuckGoSearch;
impl DuckDuckGoSearch {
    pub fn get_links(&self, search_term: &str) -> Result<Vec<Link>, IsError> {
        scrape(&format!(
            "https://html.duckduckgo.com/html/?q={}",
            &search_term
        ))
        .and_then(|html| self.links_from_html(&html))
    }

    fn links_from_html(&self, html: &str) -> Result<Vec<Link>, IsError> {
        let document = Html::parse_document(html);
        let selector_title = Selector::parse("a.result__a")
            .map_err(|_| SelectorError(String::from("Failed to create title selector")))?;
        let selector_url = Selector::parse("a.result__url")
            .map_err(|_| SelectorError(String::from("Failed to create url selector")))?;
        Ok(document
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
            .collect())
    }
}

impl Search for DuckDuckGoSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        self.get_links(query)
    }
}
