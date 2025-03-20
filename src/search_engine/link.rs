#[cfg(not(test))]
use crate::config::load::Config;
use crate::config::site::SiteConfig;

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

    #[cfg(not(test))]
    pub fn get_config(&self) -> SiteConfig {
        Config::get_site_config(self.get_url()).clone()
    }
}

#[derive(Clone)]
pub struct Link {
    pub url: String,
}
impl Link {
    pub fn new(url: &str) -> Self {
        Self {
            url: Self::format_url(url),
        }
    }

    fn format_url(url: &str) -> String {
        if url.starts_with("http") {
            return url.to_string();
        }
        format!("https://{url}")
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
#[cfg(test)]
pub mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use parking_lot::RwLock;

    pub static TEST_CONFIG: Lazy<RwLock<SiteConfig>> =
        Lazy::new(|| RwLock::new(SiteConfig::default()));

    impl HtmlSource {
        #[cfg(test)]
        pub fn get_config(&self) -> SiteConfig {
            TEST_CONFIG.read().clone()
        }
    }
}
