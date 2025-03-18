use crate::app::enum_values::HistoryViewer;
use crate::app::event_loop::{history_event_loop, HistoryAction};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::database::history_database::{get_history, HistoryData};
use crate::pipe::history::pipe_history;
use crate::search_engine::link::HtmlSource::LinkSource;
use crate::search_engine::link::{HtmlSource, Link};
use crate::tui::history_content::HistoryContent;
use ratatui::widgets::TableState;

#[derive(Clone)]
pub enum SearchOn {
    Title,
    Url,
}

impl HistoryViewer for TuiApp {
    fn show_history(&mut self) -> Option<HtmlSource> {
        let history =
            get_history().unwrap_or_else(|_| self.display.shutdown_with_error("No history found."));
        let table_state = TableState::default();
        let mut history_content = HistoryContent::new(
            history,
            String::new(),
            SearchOn::Title,
            self.display.area(),
            table_state,
        );
        {
            self.display
                .render(history_content.create_widgets(self.display.area()));
        }
        loop {
            match history_event_loop() {
                HistoryAction::Continue => continue,
                HistoryAction::Exit => {
                    self.display.shutdown();
                    break;
                }
                HistoryAction::Open => {
                    return current_link(
                        &history_content.current_history,
                        &history_content.table_state,
                    );
                }
                HistoryAction::Up => history_content.scroll_up(),
                HistoryAction::Down => history_content.scroll_down(),
                HistoryAction::Delete => history_content.remove_current(),
                HistoryAction::Text(char) => {
                    history_content.add_char(char);
                }
                HistoryAction::Backspace => history_content.remove_char(),
                HistoryAction::ChangeSearch => {
                    history_content.change_search();
                }
            }
            {
                self.display
                    .render(history_content.create_widgets(self.display.area()));
            }
        }
        None
    }
}

impl HistoryViewer for TextApp {
    fn show_history(&mut self) -> Option<HtmlSource> {
        let history =
            get_history().unwrap_or_else(|_| Self::terminating_error("Cannot access history."));
        pipe_history(history).unwrap_or_else(|_| eprintln!("Pipe broken!"));
        None
    }
}

fn current_link(history: &[HistoryData], state: &TableState) -> Option<HtmlSource> {
    let idx = state.selected().unwrap_or(0);
    history
        .iter()
        .collect::<Vec<_>>()
        .get(idx)
        .map(|history_data| LinkSource(Link::new(history_data.url.clone())))
}
