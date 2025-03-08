#[derive(Clone, Debug)]
pub struct Link {
    pub url: String,
    pub title: String,
    pub selector: String,
}
impl Link {
    pub fn new(title: String, url: String, selector: String) -> Self {
        Self {
            url,
            title,
            selector,
        }
    }
}
