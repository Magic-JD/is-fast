use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use scraper::{ElementRef, Html, Selector};

pub fn filter(html: &Html, selector_tag: String) -> Result<Vec<ElementRef>, IsError> {
    let selector = Selector::parse(&selector_tag)
        .map_err(|_| General("Error: Could not parse selector".into()))?;
    Ok(html.select(&selector).collect::<Vec<ElementRef>>())
}
