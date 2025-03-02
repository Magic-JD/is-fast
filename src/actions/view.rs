use crate::formatting::format::to_display;
use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use crate::stout::pipe::out_to_std;
use crate::tui::browser::Browser;
use ratatui::text::Text;
use std::fs;

pub fn run(file: String, url: Option<String>, piped: bool) {
    let url = url.unwrap_or_else(|| file.clone());
    let html = fs::read_to_string(&file).unwrap();
    let link = Link::new(file, url, move || Ok(html.clone()));
    if piped {
        out_to_std(
            scrape(&link.url)
                .and_then(|html| to_display(&link.url, &html))
                .unwrap_or_else(|_| Text::from("Failed to convert to text")),
        );

        return;
    }
    Browser::new().browse(vec![link]);
}
