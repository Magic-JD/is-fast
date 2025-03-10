use crate::app::history::SearchOn;
use crate::database::connect::HistoryData;
use crate::tui::display::Widget;
use crate::tui::display::Widget::{Block, Paragraph, Table, Text};
use crate::tui::general_widgets::default_block;
use crate::tui::history_widgets::{create_table, draw_history_count, draw_search_text};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::TableState;

pub(crate) static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete | Tab: Change search ";

pub fn displayables<'a>(
    state: &'a mut TableState,
    current_history: &'a [HistoryData],
    search_term: &'a str,
    search_on: &'a SearchOn,
    available_space: Rect,
) -> Vec<Widget<'a>> {
    let table = create_table(current_history, search_term, search_on);
    let block = default_block(" History ", HISTORY_INSTRUCTIONS);
    let search = draw_search_text(search_term, search_on);
    let row_count = draw_history_count(current_history.len() as u16);
    let (border_area, table_area, search_area, count_row_area) =
        history_areas(available_space, current_history.len() as u16);
    vec![
        Block(block, border_area),
        Table(table, state, table_area),
        Paragraph(search, search_area),
        Text(row_count, count_row_area),
    ]
}

fn history_areas(size: Rect, row_count: u16) -> (Rect, Rect, Rect, Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Min(1),
                Constraint::Length(row_count.min(size.height)),
                Constraint::Length(2),
            ]
            .as_ref(),
        );
    let areas = layout.split(size);
    let search_bar_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
        .split(areas[2]);

    let history_rows = areas[1];
    let search_text = search_bar_layout[0];
    let history_count = search_bar_layout[1];
    (size, history_rows, search_text, history_count)
}
