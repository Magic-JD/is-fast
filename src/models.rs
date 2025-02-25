use crate::extract::extract_page_content;
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

        match scrape(&format!("https://{}", self.url))
            .and_then(|html| extract_page_content(&self.url, &html))
        {
            Ok(result) => {
                *content = Some(result.clone());
                result
            }
            Err(_) => {
                // It may be that reqwest is not able to access the site, so fallback to curl.
                match fallback_curl(&format!("https://{}", self.url))
                    .and_then(|html| extract_page_content(&self.url, &html))
                {
                    Ok(result) => {
                        *content = Some(result.clone());
                        result
                    }
                    Err(e) => {
                        let error_string = e.clone();
                        Paragraph::new(Text::from(error_string)) // The error will be displayed on the page
                    }
                }
            }
        }
    }
}
