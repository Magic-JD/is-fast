use crate::app::enum_values::PageViewer;
use crate::app::event_loop::{page_event_loop, PageAction};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::config::load::{Config, Scroll};
use crate::database::history_database::add_history;
use crate::search_engine::link::PageSource;
use crate::transform::pretty_print::conditional_formatting;
use crate::tui::page_content::PageContent;

impl PageViewer for TuiApp {
    fn show_pages(&mut self, pages: &[PageSource]) {
        let height = self.display.height() - 2; // Subtract for the border
        let mut scroll: u16 = 0;
        if pages.is_empty() {
            self.display.shutdown_with_error("No results found.");
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
                PageAction::PageUp => match Config::get_scroll() {
                    Scroll::Full => scroll = scroll.saturating_sub(height),
                    Scroll::Half => scroll = scroll.saturating_sub(height / 2),
                    Scroll::Discrete(amount) => scroll = scroll.saturating_sub(*amount),
                },
                PageAction::PageDown => match Config::get_scroll() {
                    Scroll::Full => scroll = scroll.saturating_add(height),
                    Scroll::Half => scroll = scroll.saturating_add(height / 2),
                    Scroll::Discrete(amount) => scroll = scroll.saturating_add(*amount),
                },
                PageAction::Open => {
                    self.open_link(index, pages)
                        .unwrap_or_else(|err| self.display.shutdown_with_error(&err.to_string()));
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
                let (title, content) = page.extract.get_text(&page.html_source);
                if page.tracked {
                    let url = page.html_source.get_url();
                    add_history(&title, url).unwrap_or_else(|err| {
                        log::error!("Failed to add history for page {title} ({url}) {err}");
                    });
                }
                log::debug!("Outputting page {title} to terminal");
                println!(
                    "{}",
                    conditional_formatting(content, Config::get_pretty_print())
                );
            }
            [] => eprintln!("No links found, no error detected."),
        }
    }
}
