use crate::app::enum_values::HistoryViewer;
use crate::app::event_loop::{history_event_loop, HistoryAction};
use crate::app::history::SearchOn::{Title, Url};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::config::load::Config;
use crate::database::connect::{get_history, remove_history, HistoryData};
use crate::pipe::history::pipe_history;
use crate::search_engine::link::{Link, PageSource};
use crate::transform::page::PageExtractor;
use crate::tui::history_content::displayables;
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};
use once_cell::sync::Lazy;
use ratatui::widgets::TableState;
use std::cmp::Ordering;

static SEARCH_TYPE: Lazy<&AtomKind> = Lazy::new(Config::get_search_type);

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
    fn show_history(&mut self) -> Option<PageSource> {
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
        self.display.render(displayables(
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
                    return current_link(&state.current_history, &state.table_state).map(|link| {
                        PageSource {
                            link,
                            extract: PageExtractor::from_url(),
                            tracked: true,
                        }
                    });
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
            self.display.render(displayables(
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
    fn show_history(&mut self) -> Option<PageSource> {
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

pub fn order_by_match(
    history: &mut [HistoryData],
    user_search: &mut String,
    search_on: &SearchOn,
) -> Vec<HistoryData> {
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        &*user_search,
        CaseMatching::Ignore,
        Normalization::Smart,
        **SEARCH_TYPE,
    );
    let mut data_2_score = history
        .iter()
        .map(|h| {
            let match_on = search_on_history(h, search_on);
            (
                h,
                pattern.score(Utf32Str::new(match_on, &mut vec![]), &mut matcher),
            )
        })
        .filter(|(_, score)| score.is_some())
        .collect::<Vec<(&HistoryData, Option<u32>)>>();
    data_2_score.sort_by(|(h1, a), (h2, b)| {
        match a.unwrap_or_else(|| 0).cmp(&b.unwrap_or_else(|| 0)) {
            Ordering::Less => Ordering::Less,
            Ordering::Equal => h1.time.cmp(&h2.time),
            Ordering::Greater => Ordering::Greater,
        }
    });
    data_2_score.into_iter().map(|(a, _)| a.clone()).collect()
}

fn search_on_history<'a>(history: &'a HistoryData, search_on: &'a SearchOn) -> &'a str {
    match search_on {
        SearchOn::Title => &history.title,
        SearchOn::Url => &history.url,
    }
}
#[cfg(test)]
mod tests {
    use crate::app::history::{order_by_match, SearchOn};
    use crate::database::connect::HistoryData;
    use chrono::NaiveDateTime;

    #[test]
    fn test_order_by_match_title() {
        let mut history = vec![
            HistoryData {
                title: "Rust programming".to_string(),
                url: "https://rust-lang.org".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Rustacean".to_string(),
                url: "https://rustacean.net".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:05:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Programming in Rust".to_string(),
                url: "https://example.com".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:10:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Java is the best".to_string(),
                url: "https://example.com".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:10:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "R U S T is great".to_string(),
                url: "https://example.com".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:15:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
        ];
        let mut search_query = "Rust".to_string();
        let results = order_by_match(&mut history, &mut search_query, &SearchOn::Title);

        assert_eq!(results.len(), 4);
        assert_eq!(results[0].title, "R U S T is great");
        assert_eq!(results[1].title, "Rust programming");
        assert_eq!(results[2].title, "Rustacean");
        assert_eq!(results[3].title, "Programming in Rust");
    }

    #[test]
    fn test_order_by_match_url() {
        let mut history = vec![
            HistoryData {
                title: "Random site".to_string(),
                url: "https://example.com/rust".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:15:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Another site".to_string(),
                url: "https://rustacean.net".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:05:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Java site".to_string(),
                url: "https://java.net".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:05:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Scattered letters".to_string(),
                url: "https://example.com/r/u/s/t".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:20:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
        ];
        let mut search_query = "rust".to_string();
        let results = order_by_match(&mut history, &mut search_query, &SearchOn::Url);

        assert_eq!(results.len(), 3);
        assert_eq!(results[0].url, "https://example.com/r/u/s/t");
        assert_eq!(results[1].url, "https://rustacean.net");
        assert_eq!(results[2].url, "https://example.com/rust");
    }

    #[test]
    fn test_order_by_match_time_tiebreaker() {
        let mut history = vec![
            HistoryData {
                title: "Rust Guide".to_string(),
                url: "https://guide.rust-lang.org".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:00:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
            HistoryData {
                title: "Rust Reference".to_string(),
                url: "https://doc.rust-lang.org/reference".to_string(),
                time: NaiveDateTime::parse_from_str("2023-10-01 12:30:00", "%Y-%m-%d %H:%M:%S")
                    .unwrap(),
            },
        ];
        let mut search_query = "Rust".to_string();
        let results = order_by_match(&mut history, &mut search_query, &SearchOn::Title);

        assert_eq!(results.len(), 2);
        assert!(results[0].time < results[1].time);
    }
}
