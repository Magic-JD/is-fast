use crate::formatting::format::to_display;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use std::sync::Mutex;

pub struct Link {
    pub url: String,
    pub title: String,
    pub convert_to_html: Box<dyn Fn() -> Result<String, String>>,
    pub content: Mutex<Option<Paragraph<'static>>>,
}
impl Link {
    pub fn new<F>(title: String, url: String, convert_to_html: F) -> Self
    where
        F: Fn() -> Result<String, String> + 'static
    {
        Self {
            url,
            title,
            convert_to_html: Box::new(convert_to_html),
            content: Mutex::new(None),
        }
    }

    pub fn get_content(&self) -> Paragraph<'static> {
        let mut content = self.content.lock().unwrap();
        if let Some(ref cached) = *content {
            return cached.clone();
        }
        (self.convert_to_html)()
            .and_then(|html| to_display(&self.url, &html))
            .and_then(|result| {
                *content = Some(result.clone());
                Ok(result)
            })
            .unwrap_or_else(|e| Paragraph::new(Text::from(e.clone())).into())
    }
}
