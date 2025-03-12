use crate::cli::command::Cli;
use crate::config::load::Config;
use crate::database::connect::get_latest_history;
use crate::errors::error::IsError;
use crate::search_engine::link::{Link, PageSource};
use crate::search_engine::search::find_links;
use crate::transform::page::PageExtractor;

pub fn prepare_pages(args: Cli) -> Result<Vec<PageSource>, IsError> {
    let mut pages = vec![];
    let history_enabled = Config::get_history_enabled();
    if args.last {
        if let Some(history) = get_latest_history()? {
            pages.push(PageSource {
                link: Link::new(history.title, history.url),
                extract: PageExtractor::from_url(
                    args.color
                        .clone()
                        .unwrap_or_else(|| Config::get_color_mode().clone()),
                    args.selector.clone(),
                    args.element_nth.clone(),
                ),
                tracked: true,
            })
        }
    }
    if let Some(file) = args.file {
        let url = args.url.unwrap_or_else(|| file.clone());
        let link = Link::new(file, url);
        pages.push(PageSource {
            link,
            extract: PageExtractor::from_file(
                args.color
                    .clone()
                    .unwrap_or_else(|| Config::get_color_mode().clone()),
                args.selector.clone(),
                args.element_nth.clone(),
            ),
            tracked: false, // Must check history enabled if this changes.
        });
    }
    for url in args.direct {
        let selection_tag = args.selector.clone();
        let link = Link::new(String::default(), url);
        pages.push(PageSource {
            link,
            extract: PageExtractor::from_url(
                args.color
                    .clone()
                    .unwrap_or_else(|| Config::get_color_mode().clone()),
                selection_tag,
                args.element_nth.clone(),
            ),
            tracked: false, // Must check history enabled if this changes.
        });
    }
    if let Some(search_term) = args.query.map(|query| query.join(" ")) {
        let site = args
            .site
            .or_else(|| Config::get_site().clone())
            .map(|s| format!("site:{s}"))
            .unwrap_or_default();
        let links_result = find_links(&format!("{search_term} {site}"));
        let new_pages: Vec<PageSource> = links_result.map(|links| {
            links
                .into_iter()
                .map(|link| PageSource {
                    link,
                    extract: PageExtractor::from_url(
                        args.color
                            .clone()
                            .unwrap_or_else(|| Config::get_color_mode().clone()),
                        args.selector.clone(),
                        args.element_nth.clone(),
                    ),
                    tracked: *history_enabled,
                })
                .collect()
        })?;
        for page in new_pages {
            pages.push(page);
        }
    }
    Ok(pages)
}
