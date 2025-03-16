use crate::search_engine::link::HtmlSource;
use crate::transform::page::PageExtractor;
use dashmap::DashMap;
use once_cell::sync::{Lazy, OnceCell};
use ratatui::widgets::Paragraph;
use std::thread;

type CachedContent = OnceCell<(String, Paragraph<'static>)>;

static CACHE: Lazy<DashMap<String, CachedContent>> = Lazy::new(DashMap::new);

pub fn get_content(
    html_source: &HtmlSource,
    extractor: &PageExtractor,
) -> (String, Paragraph<'static>) {
    let identifier = match html_source {
        HtmlSource::LinkSource(link) => &link.url,
        HtmlSource::FileSource(file) => &file.file_path,
    };
    CACHE
        .entry(identifier.clone())
        .or_default()
        .get_or_init(|| extractor.get_paragraph(html_source))
        .clone()
}

pub fn preload(html_source: &HtmlSource, extractor: &PageExtractor) {
    let source_c = html_source.clone();
    let extractor_c = extractor.clone();
    thread::spawn(move || {
        _ = get_content(&source_c, &extractor_c); // Dont use the value here, just retrieve to preload.
    });
}
