use crate::formatting::format::{to_display, to_error_display, UIComponent};
use crate::scrapers::scrape::{curl_scrape, reqwest_scrape};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use std::sync::Mutex;

pub struct Link {
    pub url: String,
    pub title: String,
    pub content: Mutex<Vec<UIComponent>>,
}
impl Link {
    pub fn new(title: String, url: String) -> Self {
        Self {
            url,
            title,
            content: Mutex::new(vec![]),
        }
    }

    pub fn get_content(&self) -> Vec<UIComponent> {
        let mut content = self.content.lock().unwrap();
        if !content.is_empty() {
            return content.clone()
        }
        let formatted_url = &format!("https://{}", self.url);
        reqwest_scrape(formatted_url)
            .and_then(|html| to_display(&self.url, &html))
            .and_then(|result| {
                *content = result.clone();
                Ok(result)
            })
            .unwrap_or_else(|_| curl_scrape(formatted_url) // Try with curl on failure
                .and_then(|html| to_display(&self.url, &html))
                .and_then(|result| {
                *content = result.clone();
                Ok(result)
            }).unwrap_or_else(|e| to_error_display(e)))
    }
}
