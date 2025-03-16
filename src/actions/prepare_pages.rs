use crate::cli::command::Cli;
use crate::config::load::Config;
use crate::database::history_database::get_latest_history;
use crate::errors::error::IsError;
use crate::search_engine::link::HtmlSource::{FileSource, LinkSource};
use crate::search_engine::link::{File, Link, PageSource};
use crate::search_engine::search::find_links;
use crate::transform::page::PageExtractor;

pub fn prepare_pages(args: Cli) -> Result<Vec<PageSource>, IsError> {
    let mut pages = vec![];
    let history_enabled = Config::get_history_enabled();
    if args.last {
        if let Some(history) = get_latest_history()? {
            pages.push(PageSource {
                html_source: LinkSource(Link::new(history.url)),
                extract: PageExtractor::new(
                    Config::get_color_mode().clone(),
                    args.selector.clone(),
                    args.nth_element.clone(),
                ),
                tracked: *history_enabled,
            });
        }
    }
    if let Some(file_location) = args.file {
        let file = File::new(file_location, args.url.unwrap_or_default());
        pages.push(PageSource {
            html_source: FileSource(file),
            extract: PageExtractor::new(
                Config::get_color_mode().clone(),
                args.selector.clone(),
                args.nth_element.clone(),
            ),
            tracked: false, // Cannot track history for file.
        });
    }
    for url in args.direct {
        let selection_tag = args.selector.clone();
        let html_source = LinkSource(Link::new(url));
        pages.push(PageSource {
            html_source,
            extract: PageExtractor::new(
                Config::get_color_mode().clone(),
                selection_tag,
                args.nth_element.clone(),
            ),
            tracked: *history_enabled,
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
                .map(LinkSource)
                .map(|html_source| PageSource {
                    html_source,
                    extract: PageExtractor::new(
                        Config::get_color_mode().clone(),
                        args.selector.clone(),
                        args.nth_element.clone(),
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
