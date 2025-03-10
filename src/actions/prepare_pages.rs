use crate::cli::command::Cli;
use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::search_engine::link::{Link, PageSource};
use crate::search_engine::search::find_links;
use crate::transform::page::PageExtractor;

pub fn prepare_pages(args: Cli) -> Result<Vec<PageSource>, IsError> {
    let mut pages = vec![];
    if let Some(file) = args.file {
        let link = link_from_file(args.url, &args.selector, file);
        pages.push(PageSource {
            link,
            extract: PageExtractor::from_file(),
            tracked: false,
        });
    }
    for url in args.direct {
        let selection_tag = args
            .selector
            .clone()
            .unwrap_or_else(|| Config::get_selectors(&url));
        let link = Link::new(String::default(), url, selection_tag);
        pages.push(PageSource {
            link,
            extract: PageExtractor::from_url(),
            tracked: false,
        });
    }
    if let Some(search_term) = args.query.map(|query| query.join(" ")) {
        let site = args
            .site
            .or_else(|| Config::get_site().clone())
            .map(|s| format!("site:{s}"))
            .unwrap_or_default();
        let links_result = find_links(&format!("{search_term}{site}"));
        let new_pages: Vec<PageSource> = links_result.map(|links| {
            links
                .into_iter()
                .map(|link| PageSource {
                    link,
                    extract: PageExtractor::from_url(),
                    tracked: true,
                })
                .collect()
        })?;
        for page in new_pages {
            pages.push(page);
        }
    }
    Ok(pages)
}

fn link_from_file(url: Option<String>, selector: &Option<String>, file: String) -> Link {
    let url = url.unwrap_or_else(|| file.clone());
    let selection_tag = selector
        .clone()
        .unwrap_or_else(|| Config::get_selectors(&url));
    Link::new(file, url, selection_tag)
}
