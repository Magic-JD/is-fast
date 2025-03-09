use crate::actions::search::find_links;
use crate::app::enum_values::PageViewer;
use crate::app::event_loop::{page_event_loop, PageAction};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::cli::command::Cli;
use crate::config::load::Config;
use crate::search::link::{Extractable, Link};
use crate::search::scrape::format_url;
use crate::transform::page::PageExtractor;
use crate::tui::browser::displayables;

impl PageViewer for TuiApp {
    fn show_page(&mut self, args: Cli) {
        let extractables = create_extractables(args);
        let height = self.display.height();
        let mut scroll: u16 = 0;
        if extractables.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        self.display.loading();
        let displayables = displayables(index, scroll, &extractables, self.display.area());
        self.display.draw(displayables);
        loop {
            match page_event_loop() {
                PageAction::Exit => break,
                PageAction::Next => {
                    if index < extractables.len() - 1 {
                        self.display.loading();
                        scroll = 0;
                        index += 1;
                    }
                }
                PageAction::Previous => {
                    if index > 0 {
                        self.display.loading();
                        scroll = 0;
                        index -= 1;
                    }
                }
                PageAction::Down => {
                    scroll = scroll.saturating_add(1);
                }
                PageAction::Up => {
                    scroll = scroll.saturating_sub(1);
                }
                PageAction::PageUp => {
                    scroll = scroll.saturating_sub(height / 2);
                }
                PageAction::PageDown => {
                    scroll = scroll.saturating_add(height / 2);
                }
                PageAction::Open => {
                    open_link(index, &extractables);
                }
                PageAction::Continue => continue,
            }
            let displayables = crate::tui::browser::displayables(
                index,
                scroll,
                &extractables,
                self.display.area(),
            );
            self.display.draw(displayables);
        }
        self.display.shutdown();
    }
}

impl PageViewer for TextApp {
    fn show_page(&mut self, args: Cli) {
        let extractables = create_extractables(args);
        match &extractables[..] {
            [extractable, ..] => {
                let content = extractable.extract.get_plain_text(&extractable.link);
                println!("{}", content);
            }
            [] => eprintln!("No links found, no error detected."),
        }
    }
}

fn create_extractables(args: Cli) -> Vec<Extractable> {
    let mut links = vec![];
    if let Some(file) = args.file {
        let link = extractable_from_file(args.url, &args.selector, file);
        links.push(Extractable {
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
        links.push(Extractable {
            link,
            extract: PageExtractor::from_url(),
            tracked: false,
        });
    }
    if let Some(search_term) = args.query.map(|query| query.join(" ")) {
        let links_result = find_links(&search_term);
        let new_links: Vec<Extractable> = links_result
            .map(|links| {
                links
                    .into_iter()
                    .map(|link| Extractable {
                        link,
                        extract: PageExtractor::from_url(),
                        tracked: true,
                    })
                    .collect()
            })
            .unwrap_or_else(|_| vec![]);
        for link in new_links {
            links.push(link);
        }
    }
    links
}

fn extractable_from_file(url: Option<String>, selector: &Option<String>, file: String) -> Link {
    let url = url.unwrap_or_else(|| file.clone());
    let selection_tag = selector
        .clone()
        .unwrap_or_else(|| Config::get_selectors(&url));
    Link::new(file, url, selection_tag)
}

fn open_link(index: usize, links: &[Extractable]) {
    links
        .get(index)
        .map(|link| format_url(&link.link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{e}"));
}
