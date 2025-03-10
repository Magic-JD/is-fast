use crate::app::enum_values::PageViewer;
use crate::app::event_loop::{page_event_loop, PageAction};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::database::connect::add_history;
use crate::search_engine::link::PageSource;
use crate::search_engine::scrape::format_url;
use crate::tui::page_content::PageContent;

impl PageViewer for TuiApp {
    fn show_pages(&mut self, pages: &[PageSource]) {
        let height = self.display.height();
        let mut scroll: u16 = 0;
        if pages.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        let mut page_content = PageContent::new(pages, self.display.area());
        self.display
            .render(page_content.create_widgets(index, scroll, pages, self.display.area()));
        loop {
            match page_event_loop() {
                PageAction::Exit => break,
                PageAction::Next => {
                    if index < pages.len() - 1 {
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
                    open_link(index, pages);
                }
                PageAction::Continue => continue,
            }
            self.display.render(page_content.create_widgets(
                index,
                scroll,
                pages,
                self.display.area(),
            ));
        }
        self.display.shutdown();
    }
}

impl PageViewer for TextApp {
    fn show_pages(&mut self, pages: &[PageSource]) {
        match pages {
            [page, ..] => {
                let content = page.extract.get_plain_text(&page.link);
                if page.tracked {
                    add_history(&page.link).unwrap_or_else(|err| eprintln!("{err}"));
                }
                println!("{content}");
            }
            [] => eprintln!("No links found, no error detected."),
        }
    }
}

fn open_link(index: usize, links: &[PageSource]) {
    links
        .get(index)
        .map(|link| format_url(&link.link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{e}"));
}
