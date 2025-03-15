use crate::search_engine::link::HtmlSource;
use crate::transform::page::PageExtractor;
use dashmap::DashMap;
use once_cell::sync::Lazy;
use ratatui::widgets::Paragraph;
use std::thread;

static CACHE: Lazy<DashMap<String, (String, Paragraph)>> = Lazy::new(DashMap::new);

pub fn get_content(
    html_source: &HtmlSource,
    extractor: &PageExtractor,
) -> (String, Paragraph<'static>) {
    let identifier = match html_source {
        HtmlSource::LinkSource(link) => &link.url,
        HtmlSource::FileSource(file) => &file.file_path,
    };
    CACHE.get(identifier).map_or_else(
        || {
            let data = extractor.get_paragraph(html_source);
            CACHE.insert(identifier.clone(), data.clone());
            data
        },
        |reference| reference.value().clone(),
    )
}

pub fn preload(html_source: &HtmlSource, extractor: &PageExtractor) {
    let source_c = html_source.clone();
    let extractor_c = extractor.clone();
    thread::spawn(move || {
        get_content(&source_c, &extractor_c);
    });
}
