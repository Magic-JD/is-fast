use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use crate::tui::browser::Browser;

pub fn run(title: Option<String>, url: String) {
    let formatted_url = format_url(url);
    let links = vec![Link::new(
        title.unwrap_or_else(|| "".to_string()),
        formatted_url.to_string(),
        move || scrape(&formatted_url.to_string())
    )];
    Browser::new().browse(links);
}

fn format_url(url: String) -> String {
    if url.starts_with("http") {
        return url
    }
    format!("https://{}", url)
}