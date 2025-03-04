use crate::database::connect::add_history;
use crate::formatting::format::to_display;
use crate::links::link::Link;
use crate::tui::browser::PAGE_INSTRUCTIONS;
use crate::tui::display::default_block;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::prelude::{Color, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};
use std::thread;

static CACHE: Lazy<DashMap<String, Paragraph>> = Lazy::new(DashMap::new);

pub fn get_content(link: &Link) -> Paragraph<'static> {
    CACHE
        .get(&link.url)
        .map(|reference| reference.value().clone())
        .unwrap_or_else(|| {
            (link.convert_to_html)()
                .and_then(|html| to_display(&link.url, &html))
                .map(|result| {
                    let paragraph = Paragraph::new(result);
                    CACHE.insert(link.clone().url, paragraph.clone());
                    paragraph
                })
                .unwrap_or_else(|e| Paragraph::new(Text::from(e.to_string())))
        })
}

pub fn new_page(index: &usize, links: &[Link], history_active: bool) -> Paragraph<'static> {
    if let Some(link) = links.get(*index + 1) {
        preload(link);
    }
    links
        .get(*index)
        .inspect(|link| {
            if history_active {
                _ = add_history(link)
            }
        })
        .map(|link| (link, get_content(link)))
        .map(|(link, paragraph)| {
            let title = extract_title(link);
            let block = default_block(&title, PAGE_INSTRUCTIONS);
            paragraph
                .block(block)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: false })
                .scroll((0, 0))
        })
        .unwrap_or_else(|| Paragraph::new(Text::from(String::from("Index out of bounds"))))
}

fn extract_title(link: &Link) -> String {
    format!(" {} ({}) ", link.title, link.url)
}

fn preload(link: &Link) {
    let clone = link.clone();
    thread::spawn(move || {
        get_content(&clone);
    });
}
