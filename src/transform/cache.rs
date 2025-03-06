use crate::search::link::Link;
use crate::transform::page::PageExtractor;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::widgets::Paragraph;
use std::thread;

static CACHE: Lazy<DashMap<String, Paragraph>> = Lazy::new(DashMap::new);

pub fn get_content(link: &Link, extractor: &PageExtractor) -> Paragraph<'static> {
    CACHE
        .get(&link.url)
        .map(|reference| reference.value().clone())
        .unwrap_or_else(|| {
            let paragraph = extractor.get_paragraph(link);
            CACHE.insert(link.url.clone(), paragraph.clone());
            paragraph
        })
}

pub fn preload(link: &Link, extractor: &PageExtractor) {
    let link_c = link.clone();
    let extractor_c = extractor.clone();
    thread::spawn(move || {
        get_content(&link_c, &extractor_c);
    });
}
