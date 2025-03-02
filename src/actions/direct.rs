use crate::formatting::format::to_display;
use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use crate::stout::pipe::out_to_std;
use crate::tui::browser::Browser;
use ratatui::text::Text;

pub fn run(title: Option<String>, url: String, piped: bool) {
    let formatted_url = format_url(url);
    let link = Link::new(
        title.unwrap_or_default(),
        formatted_url.to_string(),
        move || scrape(&formatted_url.to_string()),
    );
    if piped {
        out_to_std(
            scrape(&link.url)
                .and_then(|html| to_display(&link.url, &html))
                .unwrap_or_else(|_| Text::from("Failed to convert to text")),
        );
        return;
    }
    Browser::new().browse(vec![link], false);
}

fn format_url(url: String) -> String {
    if url.starts_with("http") {
        return url;
    }
    format!("https://{}", url)
}
