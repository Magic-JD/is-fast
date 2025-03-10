use crate::database::connect::add_history;
use crate::search_engine::link::{Link, PageSource};
use crate::transform::cache::{get_content, preload};
use crate::tui::general_widgets::TUI_BORDER_COLOR;
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};

pub fn new_page(index: usize, links: &[PageSource]) -> (String, Paragraph<'static>) {
    if let Some(extractable) = links.get(index + 1) {
        let extractor = &extractable.extract;
        let link = &extractable.link;
        preload(link, extractor); // Initiate the call to get the page after this one
    }
    links
        .get(index)
        .inspect(|link| {
            if link.tracked {
                _ = add_history(&link.link);
            }
        })
        .map(|link| (link, get_content(&link.link, &link.extract)))
        .map_or_else(
            || {
                (
                    String::from("None"),
                    Paragraph::new(Text::from(String::from("Index out of bounds"))),
                )
            },
            |(link, paragraph)| {
                (
                    extract_title(&link.link),
                    paragraph
                        .style(Style::default().fg(Color::White))
                        .wrap(Wrap { trim: false })
                        .scroll((0, 0)),
                )
            },
        )
}
fn extract_title(link: &Link) -> String {
    format!(" {} ({}) ", link.title, link.url)
}

pub fn draw_page_numbers(index: usize, pages: usize) -> Text<'static> {
    Text::from(Line::styled(
        format!(" [{index}/{pages}] "),
        **TUI_BORDER_COLOR,
    ))
    .alignment(Alignment::Right)
}
