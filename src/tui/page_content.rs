use crate::config::load::Config;
use crate::search_engine::link::PageSource;
use crate::tui::display::Widget;
use crate::tui::display::Widget::{Block, Paragraph, Text};
use crate::tui::general_widgets::default_block;
use crate::tui::page_widgets::{draw_page_numbers, new_page};
use once_cell::sync::Lazy;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

static PAGE_INSTRUCTIONS: &str = " Quit: q/Esc | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o ";
static TUI_MARGIN: Lazy<u16> = Lazy::new(Config::get_page_margin);

pub fn create_widgets(
    index: usize,
    scroll: u16,
    extractables: &[PageSource],
    available_space: Rect,
) -> Vec<Widget> {
    let (title, mut page) = new_page(index, extractables);
    page = page.scroll((scroll, 0));
    let border = default_block(&title, PAGE_INSTRUCTIONS);
    let page_numbers = draw_page_numbers(index + 1, extractables.len());
    let (text_area, page_number_area) = page_area(available_space);
    vec![
        Block(border, available_space),
        Paragraph(page, text_area),
        Text(page_numbers, page_number_area),
    ]
}

pub fn page_area(size: Rect) -> (Rect, Rect) {
    //Split vertically leaving room for the header and footer.
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(size.height - 2),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(size);

    let side_margin = *TUI_MARGIN;
    let center = 100 - (side_margin * 2);

    // Split middle section horizontally to add margins to the sides.
    let horizontal_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(side_margin),
                Constraint::Percentage(center),
                Constraint::Percentage(side_margin),
            ]
            .as_ref(),
        )
        .split(vertical_chunks[1]);

    let page_number_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(vertical_chunks[2]);
    (horizontal_chunks[1], page_number_layout[1])
}
