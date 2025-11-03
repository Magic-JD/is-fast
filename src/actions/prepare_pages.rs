use crate::cli::command::OpenArgs;
use crate::config::load::Config;
use crate::database::history_database::get_latest_history;
use crate::errors::error::IsError;
use crate::search_engine::link::HtmlSource::{FileSource, LinkSource};
use crate::search_engine::link::{File, HtmlSource, Link};
use crate::search_engine::search::find_links;
use once_cell::sync::Lazy;

static SITE: Lazy<Option<String>> = Lazy::new(Config::get_search_site);

pub fn prepare_pages(query: OpenArgs) -> Result<Vec<HtmlSource>, IsError> {
    let mut sources: Vec<HtmlSource> = vec![];
    if query.last {
        if let Some(history) = get_latest_history()? {
            sources.push(LinkSource(Link::new(&history.url)));
        }
    }
    if let Some(file_location) = query.file {
        sources.push(FileSource(File::new(
            file_location,
            query.url.unwrap_or_default(),
        )));
    }
    for url in query.direct {
        sources.push(LinkSource(Link::new(&url)));
    }
    if let Some(search_term) = query.query.map(|q| q.join("+").replace(" ", "+")) {
        let site = SITE
            .clone()
            .map(|s| format!("site:{s}"))
            .unwrap_or_default();
        find_links(format!("{search_term}+{site}").trim())?
            .into_iter()
            .map(LinkSource)
            .for_each(|source| sources.push(source));
    }
    Ok(sources)
}
