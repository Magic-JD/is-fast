#[derive(Clone)]
pub enum HtmlSource {
    LinkSource(Link),
    FileSource(File),
}

impl HtmlSource {
    pub fn get_url(&self) -> &str {
        match self {
            HtmlSource::LinkSource(link) => &link.url,
            HtmlSource::FileSource(file) => &file.associated_url,
        }
    }
}

#[derive(Clone)]
pub struct Link {
    pub url: String,
}
impl Link {
    pub fn new(url: &str) -> Self {
        // Ensure that spaces are removed from scripted requests
        let url = url.replace(' ', "+");
        Self { url }
    }
}

#[derive(Clone)]
pub struct File {
    pub file_path: String,
    pub associated_url: String,
}
impl File {
    pub fn new(file: String, associated_url: String) -> Self {
        Self {
            file_path: file,
            associated_url,
        }
    }
}
