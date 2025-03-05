use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use scraper::{ElementRef, Html, Selector};

pub fn filter<'a>(html: &'a Html, selector_tag: &'a str) -> Result<Vec<ElementRef<'a>>, IsError> {
    let selector = Selector::parse(selector_tag)
        .map_err(|_| General("Error: Could not parse selector".into()))?;
    Ok(html.select(&selector).collect::<Vec<ElementRef>>())
}
