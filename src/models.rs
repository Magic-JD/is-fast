use std::sync::Mutex;
use crate::extract::extract_page_content;
use crate::scrape::scrape;

pub struct Link {
    pub url: String,
    pub title: String,
    pub content: Mutex<Option<String>>,
}
impl Link {
    pub fn new(title: String, url: String) -> Self {
        Self {
            url,
            title,
            content: Mutex::new(None),
        }
    }

    pub fn get_content(&self) -> String {
        let mut content = self.content.lock().unwrap();
        if let Some(ref cached) = *content {
            return cached.clone();
        }

        match scrape(&format!("https://{}", self.url))
            .and_then(|html| extract_page_content(&self.url, &html)) {
            Ok(result) => {
                *content = Some(result.clone());
                result
            }
            Err(e) => {
                e // The error will be displayed on the page
            }
        }
    }
}
