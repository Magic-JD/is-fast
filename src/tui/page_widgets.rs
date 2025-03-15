use crate::database::connect::add_history;
use crate::search_engine::link::PageSource;
use crate::transform::cache::{get_content, preload};
use crate::tui::general_widgets::TUI_BORDER_COLOR;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};

pub fn new_page(index: usize, links: &[PageSource]) -> (String, Paragraph<'static>) {
    if let Some(extractable) = links.get(index + 1) {
        let extractor = &extractable.extract;
        let html_source = &extractable.html_source;
        preload(html_source, extractor); // Initiate the call to get the page after this one
    }
    links.get(index).map_or_else(
        || {
            (
                String::from("None"),
                Paragraph::new(Text::from(String::from("Index out of bounds"))),
            )
        },
        |link| {
            let (title, paragraph) = get_content(&link.html_source, &link.extract);
            let url = link.html_source.get_url();
            if link.tracked {
                add_history(&title, url).unwrap_or_else(|_| {
                    log::error!("Failed to add history for page {title} ({url})")
                });
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
