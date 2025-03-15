use crate::errors::error::IsError;
use crate::errors::error::IsError::Selector as SelectorError;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::scrape;
use crate::search_engine::search_type::Search;
use scraper::{Html, Selector};

#[derive(Debug, Clone)]
pub struct DuckDuckGoSearch;
impl DuckDuckGoSearch {
    pub fn get_links(search_term: &str) -> Result<Vec<Link>, IsError> {
        scrape(&format!(
            "https://html.duckduckgo.com/html/?q={}",
            &search_term
        ))
        .and_then(|html| Self::links_from_html(&html))
    }

    fn links_from_html(html: &str) -> Result<Vec<Link>, IsError> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("div.web-result")
            .map_err(|_| SelectorError(String::from("Failed to create a result selector")))?;
        let selector_title = Selector::parse("a.result__a")
            .map_err(|_| SelectorError(String::from("Failed to create title selector")))?;
        let selector_url = Selector::parse("a.result__url")
            .map_err(|_| SelectorError(String::from("Failed to create url selector")))?;
        document
            .select(&selector)
            .map(|element_ref| Html::parse_document(&element_ref.html()))
            .map(|element_html| {
                let title = Self::extract_value(&selector_title, &element_html)?;
                let url = Self::extract_value(&selector_url, &element_html)?;
                Ok(Link::new(title, url))
            })
            .collect::<Result<Vec<Link>, IsError>>()
    }

    fn extract_value(selector: &Selector, element_html: &Html) -> Result<String, IsError> {
        let title = element_html
            .select(selector)
            .next()
            .ok_or(SelectorError(String::from("Could not parse title")))?
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
