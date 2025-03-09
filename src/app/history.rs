use crate::app::enum_values::HistoryViewer;
use crate::app::event_loop::{history_event_loop, HistoryAction};
use crate::app::history::SearchOn::{Title, Url};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::config::load::Config;
use crate::database::connect::{get_history, remove_history, HistoryData};
use crate::pipe::history::pipe_history;
use crate::search::link::Link;
use crate::tui::history::displayables;
use crate::tui::history_search::order_by_match;
use ratatui::widgets::TableState;

#[derive(Clone)]
pub(crate) struct HistoryState {
    full_history: Vec<HistoryData>,
    pub(crate) current_history: Vec<HistoryData>,
    pub(crate) search_term: String,
    pub(crate) search_on: SearchOn,
    pub(crate) table_state: TableState,
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
        let mut history_state = HistoryState {
            full_history: history.clone(),
            current_history: history.clone(),
            search_term: String::from(""),
            search_on: Title,
            table_state,
        };
        history_state.table_state.select_last();
        history_state.current_history = order_by_match(
            &mut history_state.current_history,
            &mut history_state.search_term,
            &history_state.search_on,
        );
        self.display
            .draw(displayables(&mut history_state, self.display.area()));
        loop {
            match history_event_loop() {
                HistoryAction::Continue => continue,
                HistoryAction::Exit => {
                    self.display.shutdown();
                    break;
                }
                HistoryAction::Open => {
                    return current_link(
                        &history_state.current_history,
                        &history_state.table_state,
                    );
                }
                HistoryAction::Up => {
                    if let Some(selected) = history_state.table_state.selected() {
                        history_state
                            .table_state
                            .select(Some(selected.saturating_sub(1)));
                    }
                }
                HistoryAction::Down => {
                    if let Some(selected) = history_state.table_state.selected() {
                        history_state
                            .table_state
                            .select(Some(selected.saturating_add(1).min(history.len() - 1)));
                    }
                }
                HistoryAction::Delete => {
                    let removed = history_state
                        .current_history
                        .remove(history_state.table_state.selected().unwrap_or(0));
                    _ = remove_history(&removed.url);
                    history_state.full_history.retain(|item| *item != removed);
                    *history_state.table_state.offset_mut() =
                        history_state.table_state.offset().saturating_sub(1);
                }
                HistoryAction::Text(char) => {
                    history_state.search_term.push(char);
                    history_state.current_history = order_by_match(
                        &mut history_state.current_history,
                        &mut history_state.search_term,
                        &history_state.search_on,
                    );
                    *history_state.table_state.offset_mut() = 0;
                    history_state.table_state.select_last();
                }
                HistoryAction::Backspace => {
                    history_state.search_term.pop();
                    history_state
                        .current_history
                        .clone_from(&history_state.full_history);
                    history_state.current_history = order_by_match(
                        &mut history_state.current_history,
                        &mut history_state.search_term,
                        &history_state.search_on,
                    );
                    *history_state.table_state.offset_mut() = 0;
                    history_state.table_state.select_last();
                }
                HistoryAction::ChangeSearch => {
                    history_state.search_on = next_search(&history_state.search_on);
                    history_state
                        .current_history
                        .clone_from(&history_state.full_history);
                    history_state.current_history = order_by_match(
                        &mut history_state.current_history,
                        &mut history_state.search_term,
                        &history_state.search_on,
                    );
                    *history_state.table_state.offset_mut() = 0;
                    history_state.table_state.select_last();
                }
            }
            let vec = displayables(&mut history_state, self.display.area());
            self.display.draw(vec);
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
