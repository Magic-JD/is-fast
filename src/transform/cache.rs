use crate::search_engine::link::HtmlSource;
use crate::transform::page::PageExtractor;
use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};
use ratatui::widgets::Paragraph;
use std::thread;

type CachedContent = OnceCell<(String, Paragraph<'static>)>;

static CACHE: Lazy<DashMap<String, CachedContent>> = Lazy::new(DashMap::new);

pub fn get_content(html_source: &HtmlSource) -> (String, Paragraph<'static>) {
    let identifier = identifier_for_source(html_source);
    let response = CACHE
        .entry(identifier.clone())
        .or_default()
        .get_or_init(|| PageExtractor::new().get_paragraph(html_source))
        .clone();
    log::debug!("Retrieved response for {identifier}");
    response
}

pub fn preload(html_source: &HtmlSource) {
    if CACHE.contains_key(identifier_for_source(html_source)) {
        return;
    }
    let source_c = html_source.clone();
    thread::spawn(move || {
        _ = get_content(&source_c); // Dont use the value here, just retrieve to preload.
    });
}

fn identifier_for_source(html_source: &HtmlSource) -> &String {
    match html_source {
        HtmlSource::LinkSource(link) => &link.url,
        HtmlSource::FileSource(file) => &file.file_path,
    }
}
