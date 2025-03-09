use crate::app::enum_values::HistoryViewer;
use crate::app::event_loop::{history_event_loop, HistoryAction};
use crate::app::history::SearchOn::{Title, Url};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::config::load::Config;
use crate::database::connect::{get_history, remove_history, HistoryData};
use crate::pipe::history::pipe_history;
use crate::search::link::Link;
use crate::tui::history_content::displayables;
use crate::tui::history_search::order_by_match;
use ratatui::widgets::TableState;

#[derive(Clone)]
pub(crate) struct HistoryState {
    full_history: Vec<HistoryData>,
    current_history: Vec<HistoryData>,
    search_term: String,
    search_on: SearchOn,
    table_state: TableState,
}

impl HistoryState {
    fn add_char(&mut self, c: char) {
        self.search_term.push(c);
        self.current_history = order_by_match(
            &mut self.current_history,
            &mut self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
    }

    fn remove_char(&mut self) {
        self.search_term.pop();
        self.current_history.clone_from(&self.full_history);
        self.current_history = order_by_match(
            &mut self.current_history,
            &mut self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
    }

    fn remove_current(&mut self) {
        let removed = self
            .current_history
            .remove(self.table_state.selected().unwrap_or(0));
        _ = remove_history(&removed.url);
        self.full_history.retain(|item| *item != removed);
        *self.table_state.offset_mut() = self.table_state.offset().saturating_sub(1);
    }

    fn change_search(&mut self) {
        self.search_on = next_search(&self.search_on);
        self.current_history.clone_from(&self.full_history);
        self.current_history = order_by_match(
            &mut self.current_history,
            &mut self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
    }
}

#[derive(Clone)]
pub enum SearchOn {
    Title,
    Url,
}

impl HistoryViewer for TuiApp {
    fn show_history(&mut self) -> Option<Link> {
        let history =
            get_history().unwrap_or_else(|_| self.display.shutdown_with_error("No history found."));
        let table_state = TableState::default();
        let mut state = HistoryState {
            full_history: history.clone(),
            current_history: history.clone(),
            search_term: String::from(""),
            search_on: Title,
            table_state,
        };
        state.table_state.select_last();
        state.current_history = order_by_match(
            &mut state.current_history,
            &mut state.search_term,
            &state.search_on,
        );
        self.display.draw(displayables(
            &mut state.table_state,
            &state.current_history,
            &state.search_term,
            &state.search_on,
            self.display.area(),
        ));
        loop {
            match history_event_loop() {
                HistoryAction::Continue => continue,
                HistoryAction::Exit => {
                    self.display.shutdown();
                    break;
                }
                HistoryAction::Open => {
                    return current_link(&state.current_history, &state.table_state);
                }
                HistoryAction::Up => {
                    if let Some(selected) = state.table_state.selected() {
                        state.table_state.select(Some(selected.saturating_sub(1)));
                    }
                }
                HistoryAction::Down => {
                    if let Some(selected) = state.table_state.selected() {
                        state
                            .table_state
                            .select(Some(selected.saturating_add(1).min(history.len() - 1)));
                    }
                }
                HistoryAction::Delete => state.remove_current(),
                HistoryAction::Text(char) => {
                    state.add_char(char);
                }
                HistoryAction::Backspace => state.remove_char(),
                HistoryAction::ChangeSearch => {
                    state.change_search();
                }
            }
            self.display.draw(displayables(
                &mut state.table_state,
                &state.current_history,
                &state.search_term,
                &state.search_on,
                self.display.area(),
            ));
        }
        None
    }
}

impl HistoryViewer for TextApp {
    fn show_history(&mut self) -> Option<Link> {
        let history =
            get_history().unwrap_or_else(|_| self.terminating_error("Cannot access history."));
        pipe_history(history).unwrap_or_else(|_| eprintln!("Pipe broken!"));
        None
    }
}

fn current_link(history: &[HistoryData], state: &TableState) -> Option<Link> {
    let idx = state.selected().unwrap_or(0);
    history
        .iter()
        .collect::<Vec<_>>()
        .get(idx)
        .map(|history_data| {
            Link::new(
                history_data.title.clone(),
                history_data.url.clone(),
                Config::get_selectors(&history_data.url),
            )
        })
}

fn next_search(search_on: &SearchOn) -> SearchOn {
    match search_on {
        Title => Url,
        Url => Title,
    }
}
