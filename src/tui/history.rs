use crate::actions::direct;
use crate::config::load::Config as LocalConfig;
use crate::database::connect::{remove_history, HistoryData};
use crate::tui::display::Display;
use crate::tui::history::Action::{Backspace, ChangeSearch, Continue, Down, Exit, Open, Text, Up};
use crate::tui::history::SearchOn::{Title, Url};
use chrono::{NaiveDateTime, Utc};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use nucleo_matcher::pattern::{AtomKind, CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use once_cell::sync::Lazy;
use ratatui::layout::Constraint;
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};
use std::cmp::Ordering;
use Action::Delete;

static URL_COLOR: Lazy<Style> = Lazy::new(LocalConfig::get_url_color);
static TITLE_COLOR: Lazy<Style> = Lazy::new(LocalConfig::get_title_color);
static TIME_COLOR: Lazy<Style> = Lazy::new(LocalConfig::get_time_color);
static SEARCH_TYPE: Lazy<AtomKind> = Lazy::new(LocalConfig::get_search_type);

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
        let mut state = TableState::default();
        let mut search_on = Title;
        history = order_by_match(&mut history, &mut user_search, &search_on);
        state.select(Some(history.len().saturating_sub(1)));
        let mut rows = create_rows(&history, &user_search, &search_on);
        let mut table = create_table(&mut rows);
        self.display.draw_history(
            &table,
            history.len() as u16,
            &mut state,
            &user_search,
            true,
            &search_on,
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
                    let idx = state.selected().unwrap_or(0);
                    history
                        .into_iter()
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
                    break;
                }
                Up => {
                    let state = &mut state;
                    if let Some(selected) = state.selected() {
                        if selected > 0 {
                            state.select(Some(selected - 1));
                            self.display.draw_history(
                                &table,
                                history.len() as u16,
                                state,
                                &user_search,
                                false,
                                &search_on,
                            );
                        }
                    }
                }
                Down => {
                    let state = &mut state;
                    if let Some(selected) = state.selected() {
                        if selected < (history.len() - 1) {
                            state.select(Some(selected + 1));
                            self.display.draw_history(
                                &table,
                                history.len() as u16,
                                state,
                                &user_search,
                                false,
                                &search_on,
                            );
                        }
                    }
                }
                Delete => {
                    let ref_state = &mut state;
                    let removed = history.remove(ref_state.selected().unwrap_or(0));
                    _ = remove_history(&removed.url);
                    total_history.retain(|item| *item != removed);
                    table = create_table(&mut create_rows(&history, &user_search, &search_on));
                    *state.offset_mut() = state.offset().saturating_sub(1);
                    self.display.draw_history(
                        &table,
                        history.len() as u16,
                        &mut state,
                        &user_search,
                        false,
                        &search_on,
                    );
                }
                Text(char) => {
                    user_search.push(char);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&mut create_rows(&history, &user_search, &search_on));
                    state.select(Some(history.len().saturating_sub(1)));
                    self.display.draw_history(
                        &table,
                        history.len() as u16,
                        &mut state,
                        &user_search,
                        true,
                        &search_on,
                    );
                }
                Backspace => {
                    user_search.pop();
                    history.clone_from(&total_history);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&mut create_rows(&history, &user_search, &search_on));
                    state.select(Some(history.len().saturating_sub(1)));
                    self.display.draw_history(
                        &table,
                        history.len() as u16,
                        &mut state,
                        &user_search,
                        true,
                        &search_on,
                    );
                }
                ChangeSearch => {
                    search_on = next_search(&search_on);
                    history.clone_from(&total_history);
                    history = order_by_match(&mut history, &mut user_search, &search_on);
                    table = create_table(&mut create_rows(&history, &user_search, &search_on));
                    state.select(Some(history.len().saturating_sub(1)));
                    self.display.draw_history(
                        &table,
                        history.len() as u16,
                        &mut state,
                        &user_search,
                        true,
                        &search_on,
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

fn order_by_match(
    history: &mut [HistoryData],
    user_search: &mut String,
    search_on: &SearchOn,
) -> Vec<HistoryData> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::new(
        &*user_search,
        CaseMatching::Ignore,
        Normalization::Smart,
        *SEARCH_TYPE,
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
        Title => &history.title,
        Url => &history.url,
    }
}

fn create_table<'a>(rows: &mut [Row<'a>]) -> Table<'a> {
    let table = Table::from_iter(rows.to_owned())
        .widths([
            Constraint::Percentage(50),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ])
        .column_spacing(1)
        .highlight_symbol("> ")
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
    table
}

fn create_rows(
    history: &[HistoryData],
    user_search: &str,
    search_on: &SearchOn,
) -> Vec<Row<'static>> {
    let rows: Vec<Row> = history
        .iter()
        .map(|h| match search_on {
            Title => {
                let cell = vec![
                    Cell::from(highlight_text(clip_if_needed(&h.title, 100), user_search))
                        .style(*TITLE_COLOR),
                    Cell::from(clip_if_needed(&h.url, 60)).style(*URL_COLOR),
                    Cell::from(date_to_display(&h.time)).style(*TIME_COLOR),
                ];
                Row::new(cell)
            }
            Url => {
                let cells = vec![
                    Cell::from(clip_if_needed(&h.title, 100)).style(*TITLE_COLOR),
                    Cell::from(highlight_text(clip_if_needed(&h.url, 60), user_search))
                        .style(*URL_COLOR),
                    Cell::from(date_to_display(&h.time)).style(*TIME_COLOR),
                ];
                Row::new(cells)
            }
        })
        .collect();
    rows
}

fn highlight_text(plain_text: String, user_search: &str) -> Line<'static> {
    let user_search = user_search.replace(' ', "");
    if user_search.is_empty() || plain_text.is_empty() {
        return Line::from(plain_text);
    }
    let mut matcher = Matcher::new(Config::DEFAULT);

    let mut indices = vec![];
    let mut binding1 = vec![];
    let mut binding2 = vec![];
    let haystack = Utf32Str::new(&plain_text, &mut binding1);
    let lowercase = user_search.to_lowercase(); // Panics with uppercase??? Lowercase still matches
    let needle = Utf32Str::new(&lowercase, &mut binding2);
    matcher.fuzzy_indices(haystack, needle, &mut indices);
    if indices.is_empty() {
        return Line::from(plain_text);
    }
    let mut idx = indices.remove(0);
    let mut current = String::new();
    let mut spans = vec![];
    let mut found = false;
    for (c, i) in plain_text.chars().zip(0..) {
        if found || i < idx {
            current.push(c);
        } else {
            spans.push(Span::from(current.clone()));
            current = String::new();
            spans.push(Span::styled(
                String::from(c),
                Style::from(Color::Red).add_modifier(Modifier::BOLD),
            ));
            if indices.is_empty() {
                found = true;
            } else {
                idx = indices.remove(0);
            }
        }
    }
    spans.push(Span::from(current));
    Line::from(spans)
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

fn clip_if_needed(text: &str, max_length: usize) -> String {
    if text.len() > max_length {
        return format!("{}...", &text[0..max_length - 3]);
    }
    text.to_string()
}

fn date_to_display(date: &NaiveDateTime) -> String {
    let duration = Utc::now().signed_duration_since(date.and_utc());
    if duration.num_weeks() > 0 {
        return format_time(duration.num_weeks(), "weeks");
    }
    if duration.num_days() > 0 {
        return format_time(duration.num_days(), "days");
    }
    if duration.num_hours() > 0 {
        return format_time(duration.num_hours(), "hours");
    }
    if duration.num_minutes() > 0 {
        return format_time(duration.num_minutes(), "minutes");
    }
    if duration.num_seconds() > 0 {
        return format_time(duration.num_seconds(), "seconds");
    }
    "Date could not be displayed".to_string()
}

fn format_time(amount: i64, time_measurement: &str) -> String {
    format!("{amount} {time_measurement} ago")
}
