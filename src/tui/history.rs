use crate::app::history::HistoryState;
use crate::tui::display::Widget;
use crate::tui::display::Widget::{Block, Paragraph, Table, Text};
use crate::tui::general_widgets::default_block;
use crate::tui::history_widgets::{create_table, draw_history_count, draw_search_text};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub(crate) static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete | Tab: Change search ";

pub fn displayables(state: &mut HistoryState, available_space: Rect) -> Vec<Widget> {
    let table = create_table(&state.current_history, &state.search_term, &state.search_on);
    let block = default_block(" History ", HISTORY_INSTRUCTIONS);
    let search = draw_search_text(&state.search_term, &state.search_on);
    let row_count = draw_history_count(state.current_history.len() as u16);
    let (border_area, table_area, search_area, count_row_area) =
        history_areas(available_space, state.current_history.len() as u16);
    vec![
        Block(block, border_area),
        Table(table, &mut state.table_state, table_area),
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
