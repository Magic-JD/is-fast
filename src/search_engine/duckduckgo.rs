use crate::errors::error::IsError;
use crate::errors::error::IsError::{Search as SearchError, Selector as SelectorError};
use crate::search_engine::link::HtmlSource::LinkSource;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::{cache_purge, scrape};
use crate::search_engine::search_type::Search;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DuckDuckGoSearch;
impl DuckDuckGoSearch {
    pub fn get_links(search_term: &str) -> Result<Vec<Link>, IsError> {
        let html_source = &LinkSource(Link::new(&format!(
            "https://html.duckduckgo.com/html/?q={}",
            &search_term
        )));
        scrape(html_source)
            .and_then(|html| Self::links_from_html(&html).inspect_err(|_| cache_purge(html_source)))
    }

    fn links_from_html(html: &str) -> Result<Vec<Link>, IsError> {
        let selector = Selector::parse("div.web-result")
            .map_err(|_| SelectorError(String::from("Failed to create a result selector")))?;
        let selector_url = Selector::parse("a.result__url")
            .map_err(|_| SelectorError(String::from("Failed to create url selector")))?;
        Html::parse_document(html)
            .select(&selector)
            .map(|element_ref| Html::parse_document(&element_ref.html()))
            .map(|element_html| {
                let url = Self::extract_value(&selector_url, &element_html)?;
                Ok(Link::new(&url))
            })
            .collect::<Result<Vec<Link>, IsError>>()
            .and_then(|links| {
                if links.is_empty() {
                    Err(SearchError(String::from("No links found")))
                } else {
                    Ok(links)
                }
            })
    }

    fn extract_value(selector: &Selector, element_html: &Html) -> Result<String, IsError> {
        let title = element_html
            .select(selector)
            .next()
            .ok_or(SelectorError(String::from("Could not parse")))?
            .text()
            .collect::<Vec<_>>()
            .join("")
            .trim()
            .to_string();
        Ok(title)
    }
}

impl Search for DuckDuckGoSearch {
    fn search(&self, query: &str) -> Result<Vec<Link>, IsError> {
        Self::get_links(query)
    }
}
