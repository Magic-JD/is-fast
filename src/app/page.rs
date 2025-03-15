use crate::app::enum_values::PageViewer;
use crate::app::event_loop::{page_event_loop, PageAction};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::cli::command::ColorMode;
use crate::config::load::{Config, Scroll};
use crate::database::connect::add_history;
use crate::search_engine::link::PageSource;
use crate::tui::page_content::PageContent;
use crate::DisplayConfig;
use nu_ansi_term::Style;
use terminal_size::{terminal_size, Width};
use textwrap::{fill, Options, WrapAlgorithm};

impl PageViewer for TuiApp {
    fn show_pages(&mut self, pages: &[PageSource]) {
        let height = self.display.height() - 2; // Subtract for the border
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
                if page.tracked {
                    add_history(&page.link).unwrap_or_else(|err| eprintln!("{err}"));
                }
                let content = page.extract.get_text(&page.link);
                let width = match terminal_size() {
                    Some((Width(w), _)) => w,
                    None => {
                        log::error!("Failed to get terminal size - defaulting to sane value");
                        80
                    }
                };
                println!("{}", conditional_formatting(content, width));
            }
            [] => eprintln!("No links found, no error detected."),
        }
    }
}

fn conditional_formatting(mut content: String, mut width: u16) -> String {
    let display_configuration_list = Config::get_display_configuration();
    // No additional configuration, early return.
    if display_configuration_list.is_empty() {
        return format!("\n{}\n", content.trim()); //Ensure consistent blank first and last line
    }

    let mut margin = 0;
    let mut wrap = false;
    let mut title = None;
    for display_config in display_configuration_list {
        match display_config {
            DisplayConfig::Margin(amount) => {
                margin = *amount;
                wrap = true;
            }
            // As we always wrap if there is an object in the list, we don't need to do
            // anything further here.
            DisplayConfig::Wrap => wrap = true,
            DisplayConfig::Title(title_val) => title = Some(title_val),
        }
    }
    if let Some(title_val) = title {
        let mut title = format!("\n{title_val}\n");
        if let ColorMode::Always = Config::get_color_mode() {
            let style = Style::new().bold();
            title = style.paint(&title).to_string();
        }
        content.insert_str(0, &title);
    }
    if wrap {
        let total_margin = margin * 2;
        if width > total_margin {
            width -= total_margin;
        }
        content = fill(
            &content,
            Options::new(width as usize).wrap_algorithm(WrapAlgorithm::FirstFit),
        );
        if margin > 0 {
            content = textwrap::indent(&content, &" ".repeat(margin as usize))
        }
    }
    content
}
