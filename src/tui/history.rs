use crate::actions::direct;
use crate::database::connect::{remove_history, HistoryData};
use crate::tui::display::Display;
use crate::tui::general_widgets::default_block;
use crate::tui::history::Action::{Backspace, ChangeSearch, Continue, Down, Exit, Open, Text, Up};
use crate::tui::history::SearchOn::{Title, Url};
use crate::tui::history_search::order_by_match;
use crate::tui::history_widgets::{create_table, draw_history_count, draw_search_text};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::widgets::TableState;
use Action::Delete;

static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete | Tab: Change search ";

pub struct History {
    display: Display,
}

impl History {
    pub fn new() -> Self {
        History {
            display: Display::new(),
        }
    }

    pub fn show_history(mut self, mut history: Vec<HistoryData>) {
        if history.is_empty() {
            self.display.shutdown();
            eprintln!("No history found");
            return;
        }
        let mut total_history = history.clone();
        let mut user_search = String::new();
        let state = &mut TableState::default();
        let mut search_on = Title;
        history = order_by_match(&mut history, &mut user_search, &search_on);
        state.select_last();
        let mut table = create_table(&history, &user_search, &search_on);
        let border = default_block(" History ", HISTORY_INSTRUCTIONS);
        let mut search = draw_search_text(&user_search, &search_on);
        let mut entry_count = draw_history_count(history.len() as u16);
        self.display.draw_history(
            &table,
            state,
            &entry_count,
            &search,
            &border,
            history.len() as u16,
        );
        loop {
            match handle_input() {
                Continue => {}
                Exit => {
                    self.display.shutdown();
                    break;
                }
                Open => {
                    self.display.shutdown();
                    open_browser(&history, state);
                    break;
                }
                Up => {
                    if let Some(selected) = state.selected() {
                        state.select(Some(selected.saturating_sub(1)));
                        self.display.draw_history(
                            &table,
                            state,
                            &entry_count,
                            &search,
                            &border,
                            history.len() as u16,
                        );
                    }
                }
                Down => {
                    if let Some(selected) = state.selected() {
                        state.select(Some(selected.saturating_add(1).min(history.len() - 1)));
                        self.display.draw_history(
                            &table,
                            state,
                            &entry_count,
                            &search,
                            &border,
                            history.len() as u16,
                        );
                    }
                }
                Delete => {
                    let removed = history.remove(state.selected().unwrap_or(0));
                    _ = remove_history(&removed.url);
                    total_history.retain(|item| *item != removed);
                    table = create_table(&history, &user_search, &search_on);
                    *state.offset_mut() = state.offset().saturating_sub(1);
                    entry_count = draw_history_count(history.len() as u16);
                    self.display.draw_history(
                        &table,
                        state,
                        &entry_count,
                        &search,
                        &border,
                        history.len() as u16,
                    );
                }
                Text(char) => {
                    user_search.push(char);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&history, &user_search, &search_on);
                    *state.offset_mut() = 0;
                    state.select_last();
                    search = draw_search_text(&user_search, &search_on);
                    entry_count = draw_history_count(history.len() as u16);
                    self.display.draw_history(
                        &table,
                        state,
                        &entry_count,
                        &search,
                        &border,
                        history.len() as u16,
                    );
                }
                Backspace => {
                    user_search.pop();
                    history.clone_from(&total_history);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&history, &user_search, &search_on);
                    *state.offset_mut() = 0;
                    state.select_last();
                    search = draw_search_text(&user_search, &search_on);
                    entry_count = draw_history_count(history.len() as u16);
                    self.display.draw_history(
                        &table,
                        state,
                        &entry_count,
                        &search,
                        &border,
                        history.len() as u16,
                    );
                }
                ChangeSearch => {
                    search_on = next_search(&search_on);
                    history.clone_from(&total_history);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&history, &user_search, &search_on);
                    *state.offset_mut() = 0;
                    state.select_last();
                    search = draw_search_text(&user_search, &search_on);
                    entry_count = draw_history_count(history.len() as u16);
                    self.display.draw_history(
                        &table,
                        state,
                        &entry_count,
                        &search,
                        &border,
                        history.len() as u16,
                    );
                }
            }
        }
    }
}

fn next_search(search_on: &SearchOn) -> SearchOn {
    match search_on {
        Title => Url,
        Url => Title,
    }
}

fn handle_input() -> Action {
    if let Ok(event::Event::Key(KeyEvent {
        code,
        kind: KeyEventKind::Press,
        ..
    })) = event::read()
    {
        return match code {
            KeyCode::Esc => Exit,
            KeyCode::Up => Up,
            KeyCode::Down => Down,
            KeyCode::Enter => Open,
            KeyCode::Delete => Delete,
            KeyCode::Char(char) => Text(char),
            KeyCode::Backspace => Backspace,
            KeyCode::Tab => ChangeSearch,
            _ => Continue,
        };
    }
    Continue
}

enum Action {
    Exit,
    Continue,
    Open,
    Up,
    Down,
    Delete,
    Text(char),
    Backspace,
    ChangeSearch,
}

pub enum SearchOn {
    Title,
    Url,
}

fn open_browser(history: &[HistoryData], state: &TableState) {
    let idx = state.selected().unwrap_or(0);
    history
        .iter()
        .collect::<Vec<_>>()
        .get(idx)
        .inspect(|history_data| {
            direct::run(
                Some(history_data.title.clone()),
                &history_data.url.clone(),
                None,
                false,
            );
        });
}
