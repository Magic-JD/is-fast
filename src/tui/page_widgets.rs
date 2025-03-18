use crate::config::load::Config;
use crate::database::history_database::add_history;
use crate::search_engine::link::HtmlSource;
use crate::search_engine::link::HtmlSource::LinkSource;
use crate::transform::cache::{get_content, preload};
use crate::tui::general_widgets::TUI_BORDER_COLOR;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};

pub fn new_page(index: usize, sources: &[HtmlSource]) -> (String, Paragraph<'static>) {
    if let Some(html_source) = sources.get(index + 1) {
        preload(html_source); // Initiate the call to get the page after this one
    }
    sources.get(index).map_or_else(
        || {
            (
                String::from("None"),
                Paragraph::new(Text::from(String::from("Index out of bounds"))),
            )
        },
        |source| {
            let (title, paragraph) = get_content(source);
            let url = source.get_url();
            if *Config::get_history_enabled() {
                if let LinkSource(_) = source {
                    add_history(&title, url).unwrap_or_else(|err| {
                        log::error!("Failed to add history for page {title} ({url}) {err}");
                    });
                }
            }
            let display_title = format!(" {title} ({url}) ");
            let formatted_paragraph = paragraph
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: false })
                .scroll((0, 0));
            (display_title, formatted_paragraph)
        },
    )
}

pub fn draw_page_numbers(index: usize, pages: usize) -> Text<'static> {
    Text::from(Line::styled(
        format!(" [{index}/{pages}] "),
        **TUI_BORDER_COLOR,
    ))
    .alignment(Alignment::Right)
}
