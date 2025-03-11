use crate::app::history::SearchOn;
use crate::app::history::SearchOn::{Title, Url};
use crate::config::load::Config;
use crate::database::connect::{remove_history, HistoryData};
use crate::tui::display::Widget;
use crate::tui::display::Widget::{Block, Paragraph, Table, Text};
use crate::tui::general_widgets::default_block;
use crate::tui::history_widgets::{create_table, draw_history_count, draw_search_text};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Matcher, Utf32Str};
use once_cell::sync::Lazy;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Text as RText;
use ratatui::widgets::TableState;
use ratatui::widgets::{Block as RBlock, Paragraph as RParagraph, Table as RTable};
use std::cmp::Ordering;

pub(crate) static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete | Tab: Change search ";

static SEARCH_TYPE: Lazy<&AtomKind> = Lazy::new(Config::get_search_type);

pub struct HistoryContent<'a> {
    total_area: Rect,
    widgets: (RBlock<'a>, RTable<'a>, RParagraph<'a>, RText<'a>),
    areas: (Rect, Rect, Rect, Rect),
    pub(crate) current_history: Vec<HistoryData>,
    full_history: Vec<HistoryData>,
    search_term: String,
    search_on: SearchOn,
    pub(crate) table_state: TableState,
    needs_update: bool,
}

impl HistoryContent<'_> {
    pub fn new(
        mut current_history: Vec<HistoryData>,
        search_term: String,
        search_on: SearchOn,
        total_area: Rect,
        mut table_state: TableState,
    ) -> Self {
        let block = default_block(" History ", HISTORY_INSTRUCTIONS);
        let full_history = current_history.clone();
        let current_history = order_by_match(&mut current_history, &search_term, &search_on);
        let table = create_table(&current_history, search_term.clone(), search_on.clone());
        let search = draw_search_text(search_term.clone(), search_on.clone());
        let row_count = draw_history_count(current_history.len() as u16);
        let widgets = (block, table, search, row_count);
        let areas = HistoryContent::history_areas(total_area, current_history.len() as u16);
        table_state.select_last();
        Self {
            total_area,
            widgets,
            areas,
            current_history,
            full_history,
            search_term,
            search_on,
            table_state,
            needs_update: false,
        }
    }

    pub fn create_widgets(&mut self, available_space: Rect) -> Vec<Widget> {
        if available_space != self.total_area || self.needs_update {
            self.total_area = available_space;
            self.areas = Self::history_areas(available_space, self.current_history.len() as u16);
        }
        if self.needs_update {
            self.widgets.1 = create_table(
                &self.current_history,
                self.search_term.clone(),
                self.search_on.clone(),
            );
            self.widgets.2 = draw_search_text(self.search_term.clone(), self.search_on.clone());
            self.widgets.3 = draw_history_count(self.current_history.len() as u16);
            self.needs_update = false;
        }
        let (block, table, search, row_count) = &self.widgets;
        let (border_area, table_area, search_area, count_row_area) = &self.areas;
        vec![
            Block(block, border_area),
            Table(table, &mut self.table_state, table_area),
            Paragraph(search, search_area),
            Text(row_count, count_row_area),
        ]
    }

    pub(crate) fn add_char(&mut self, c: char) {
        self.search_term.push(c);
        self.current_history = order_by_match(
            &mut self.current_history,
            &self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
        self.needs_update = true;
    }

    pub(crate) fn remove_char(&mut self) {
        self.search_term.pop();
        self.current_history.clone_from(&self.full_history);
        self.current_history = order_by_match(
            &mut self.current_history,
            &self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
        self.needs_update = true;
    }

    pub(crate) fn remove_current(&mut self) {
        if self.current_history.is_empty() {
            return;
        }
        let removed = self
            .current_history
            .remove(self.table_state.selected().unwrap_or(0));
        _ = remove_history(&removed.url);
        self.full_history.retain(|item| *item != removed);
        *self.table_state.offset_mut() = self.table_state.offset().saturating_sub(1);
        self.needs_update = true;
    }

    pub(crate) fn change_search(&mut self) {
        self.search_on = next_search(&self.search_on);
        self.current_history.clone_from(&self.full_history);
        self.current_history = order_by_match(
            &mut self.current_history,
            &self.search_term,
            &self.search_on,
        );
        *self.table_state.offset_mut() = 0;
        self.table_state.select_last();
        self.needs_update = true;
    }

    pub(crate) fn scroll_up(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            self.table_state.select(Some(selected.saturating_sub(1)));
        }
    }
    pub(crate) fn scroll_down(&mut self) {
        if let Some(selected) = self.table_state.selected() {
            self.table_state.select(Some(
                selected
                    .saturating_add(1)
                    .min(self.current_history.len() - 1),
            ));
        }
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
}

fn next_search(search_on: &SearchOn) -> SearchOn {
    match search_on {
        Title => Url,
        Url => Title,
    }
}
fn order_by_match(
    history: &mut [HistoryData],
    user_search: &str,
    search_on: &SearchOn,
) -> Vec<HistoryData> {
    let mut matcher = Matcher::new(nucleo_matcher::Config::DEFAULT);
    let pattern = Pattern::new(
        user_search,
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
    use crate::app::history::SearchOn;
    use crate::database::connect::HistoryData;
    use crate::tui::history_content::order_by_match;
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
        let search_query = "Rust".to_string();
        let results = order_by_match(&mut history, &search_query, &SearchOn::Title);

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
        let search_query = "rust".to_string();
        let results = order_by_match(&mut history, &search_query, &SearchOn::Url);

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
        let search_query = "Rust".to_string();
        let results = order_by_match(&mut history, &search_query, &SearchOn::Title);

        assert_eq!(results.len(), 2);
        assert!(results[0].time < results[1].time);
    }
}
