use crate::config::load::Config;
use crate::database::history_database::get_latest_history;
use crate::errors::error::IsError;
use crate::search_engine::link::HtmlSource::{FileSource, LinkSource};
use crate::search_engine::link::{File, HtmlSource, Link};
use crate::search_engine::search::find_links;

pub fn prepare_pages(
    last: bool,
    file: Option<String>,
    url: Option<String>,
    direct: Vec<String>,
    query: Option<String>,
    site: Option<String>,
) -> Result<Vec<HtmlSource>, IsError> {
    let mut sources: Vec<HtmlSource> = vec![];
    if last {
        if let Some(history) = get_latest_history()? {
            sources.push(LinkSource(Link::new(history.url)));
        }
    }
    if let Some(file_location) = file {
        sources.push(FileSource(File::new(
            file_location,
            url.unwrap_or_default(),
        )));
    }
    for url in direct {
        sources.push(LinkSource(Link::new(url)));
    }
    if let Some(search_term) = query {
        let site = site
            .or_else(|| Config::get_site().clone())
            .map(|s| format!("site:{s}"))
            .unwrap_or_default();
        find_links(format!("{search_term} {site}").trim())?
            .into_iter()
            .map(LinkSource)
            .for_each(|source| sources.push(source));
    }
    Ok(sources)
}
