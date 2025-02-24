use crate::error::MyError;
use crate::extract::extract_page_content;
use crate::scrape::scrape;

pub fn fetch_url(link: Option<&crate::models::Link>) -> Result<String, MyError> {
    let first_url = match link {
        Some(link) => link.url.clone(),
        None => return Err(MyError::DisplayError("No links available".to_string())),
    };
    scrape(&format!("https://{}", first_url))
        .and_then(|html| extract_page_content(&first_url, &html)).map_err(|e| MyError::DisplayError(e.to_string()))
}

