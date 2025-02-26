use crate::extract_formatted::extract_page_content;
use crate::scrape::{fallback_curl, scrape};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use std::sync::Mutex;

pub struct Link {
    pub url: String,
    pub title: String,
    pub content: Mutex<Option<Paragraph<'static>>>,
}
impl Link {
    pub fn new(title: String, url: String) -> Self {
        Self {
            url,
            title,
            content: Mutex::new(None),
        }
    }

    pub fn get_content(&self) -> Paragraph<'static> {
        let mut content = self.content.lock().unwrap();
        if let Some(ref cached) = *content {
            return cached.clone();
        }
        let formatted_url = &format!("https://{}", self.url);
        scrape(formatted_url)
            .and_then(|html| extract_page_content(&self.url, &html))
            .and_then(|result| {
                *content = Some(result.clone());
                Ok(result)
            })
            .unwrap_or_else(|_| fallback_curl(formatted_url) // Try with curl on failure
                .and_then(|html| extract_page_content(&self.url, &html))
                .and_then(|result| {
                *content = Some(result.clone());
                Ok(result)
            }).unwrap_or_else(|e| Paragraph::new(Text::from(e.clone())).into()))
    }
}
