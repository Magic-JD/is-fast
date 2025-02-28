use std::sync::Arc;

#[derive(Clone)]
pub struct Link {
    pub url: String,
    pub title: String,
    pub convert_to_html: Arc<dyn Fn() -> Result<String, String> + Send + Sync + 'static>,
}
impl Link {
    pub fn new<F>(title: String, url: String, convert_to_html: F) -> Self
    where
        F: Fn() -> Result<String, String> + Send + Sync + 'static
    {
        Self {
            url,
            title,
            convert_to_html: Arc::new(convert_to_html),
        }
    }
}
