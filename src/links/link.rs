use crate::formatting::format::to_display;
use crate::scrapers::scrape::{curl_scrape, reqwest_scrape};
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
        reqwest_scrape(formatted_url)
            .and_then(|html| to_display(&self.url, &html))
            .and_then(|result| {
                *content = Some(result.clone());
                Ok(result)
            })
            .unwrap_or_else(|_| curl_scrape(formatted_url) // Try with curl on failure
                .and_then(|html| to_display(&self.url, &html))
                .and_then(|result| {
                *content = Some(result.clone());
                Ok(result)
            }).unwrap_or_else(|e| Paragraph::new(Text::from(e.clone())).into()))
    }
}
