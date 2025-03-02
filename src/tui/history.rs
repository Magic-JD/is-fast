use crate::actions::direct;
use crate::database::connect::{remove_history, HistoryData};
use crate::tui::display::Display;
use crate::tui::history::Action::{Backspace, Continue, Down, Exit, Open, Text, Up};
use chrono::{NaiveDateTime, Utc};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent};
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher, Utf32Str};
use ratatui::layout::Constraint;
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Cell, Row, Table, TableState};
use std::cmp::Ordering;
use Action::Delete;

const INSTRUCTIONS: &'static str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete ";

pub struct History {
    display: Display,
}

impl History {
    pub fn new() -> Self {
        History {
            display: Display::new(INSTRUCTIONS.to_string()),
        }
    }

    pub fn show_history(mut self, mut history: Vec<HistoryData>) {
        if history.is_empty() {
            self.display.shutdown();
            eprintln!("No history found");
            return;
        }
        let mut total_history = history.clone();
        let mut user_search = String::from("");
        let mut state = TableState::default();
        history = order_by_match(&mut history, &mut user_search);
        state.select(Some(history.len().saturating_sub(1)));
        let mut rows = create_rows(history.clone(), &user_search);
        let mut table = create_table(&mut rows);
        self.display
            .draw_table(
                &table,
                history.len() as u16,
                "History".to_string(),
                &mut state,
                &mut user_search,
                true,
            )
            .expect("TODO: panic message");
        loop {
            match handle_input() {
                Continue => {}
                Exit => {
                    self.display.shutdown();
                    break;
                }
                Open => {
                    self.display.shutdown();
                    let idx = state.selected().unwrap_or_else(|| 0);
                    history
                        .into_iter()
                        .collect::<Vec<_>>()
                        .get(idx)
                        .map(HistoryData::clone)
                        .inspect(|history_data| {
                            direct::run(
                                Some(history_data.title.clone()),
                                history_data.url.clone(),
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
                            _ = self.display.draw_table(
                                &table,
                                history.len() as u16,
                                "History".to_string(),
                                state,
                                &mut user_search,
                                false,
                            );
                        }
                    }
                }
                Down => {
                    let state = &mut state;
                    if let Some(selected) = state.selected() {
                        if selected < (history.len() - 1) as usize {
                            state.select(Some(selected + 1));
                            _ = self.display.draw_table(
                                &table,
                                history.len() as u16,
                                "History".to_string(),
                                state,
                                &mut user_search,
                                false,
                            );
                        }
                    }
                }
                Delete => {
                    let ref_state = &mut state;
                    let removed = history.remove(ref_state.selected().unwrap_or_else(|| 0));
                    _ = remove_history(&removed.url);
                    total_history.retain(|item| *item != removed);
                    table = create_table(&mut create_rows(history.clone(), &user_search));
                    self.display
                        .draw_table(
                            &table,
                            history.len() as u16,
                            "History".to_string(),
                            &mut state,
                            &mut user_search,
                            false,
                        )
                        .expect("TODO: panic message");
                }
                Text(char) => {
                    user_search.push(char);
                    history = order_by_match(&mut history, &mut user_search);
                    table = create_table(&mut create_rows(history.clone(), &user_search));
                    state.select(Some(history.len().saturating_sub(1)));
                    _ = self.display.draw_table(
                        &table,
                        history.len() as u16,
                        "History".to_string(),
                        &mut state,
                        &mut user_search,
                        true,
                    );
                }
                Backspace => {
                    user_search.pop();
                    history = total_history.clone();
                    history = order_by_match(&mut history, &mut user_search);
                    table = create_table(&mut create_rows(history.clone(), &user_search));
                    state.select(Some(history.len().saturating_sub(1)));
                    _ = self.display.draw_table(
                        &table,
                        history.len() as u16,
                        "History".to_string(),
                        &mut state,
                        &mut user_search,
                        true,
                    );
                }
            }
        }
    }
}

fn order_by_match(history: &mut Vec<HistoryData>, user_search: &mut String) -> Vec<HistoryData> {
    let mut matcher = Matcher::new(Config::DEFAULT);
    let pattern = Pattern::parse(&*user_search, CaseMatching::Ignore, Normalization::Smart);
    let mut data_2_score = history
        .iter()
        .map(|h| {
            (
                h,
                pattern.score(Utf32Str::new(&*h.title, &mut vec![]), &mut matcher),
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

fn create_table<'a>(rows: &mut Vec<Row<'a>>) -> Table<'a> {
    let table = Table::from_iter(rows.clone())
        .widths(&[
            Constraint::Percentage(50),
            Constraint::Percentage(40),
            Constraint::Percentage(10),
        ])
        .column_spacing(1)
        .highlight_symbol("> ")
        .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
    table
}

fn create_rows(history: Vec<HistoryData>, user_search: &String) -> Vec<Row<'static>> {
    let rows: Vec<Row> = history
        .iter()
        .map(|h| {
            let cells = vec![
                Cell::from(highlight_title(
                    clip_if_needed(h.title.clone(), 100),
                    user_search.clone(),
                ))
                .style(Style::default().fg(Color::Yellow)),
                Cell::from(clip_if_needed(h.url.clone(), 60))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(date_to_display(h.time.clone())).style(Style::default().fg(Color::Cyan)),
            ];
            Row::new(cells)
        })
        .collect();
    rows
}

fn highlight_title(plain_text: String, user_search: String) -> Line<'static> {
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
    for (c, i) in plain_text.chars().into_iter().zip(0..) {
        if found {
            current.push(char::try_from(c).unwrap());
        } else if i < idx {
            current.push(char::try_from(c).unwrap());
        } else {
            spans.push(Span::from(current.clone()));
            current = String::new();
            spans.push(Span::styled(
                String::from(char::try_from(c).unwrap()),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
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
    event::read()
        .map(|event| {
            if let event::Event::Key(KeyEvent { code, .. }) = event {
                return match code {
                    KeyCode::Esc => Exit,
                    KeyCode::Up => Up,
                    KeyCode::Down => Down,
                    KeyCode::Enter => Open,
                    KeyCode::Delete => Delete,
                    KeyCode::Char(char) => Text(char),
                    KeyCode::Backspace => Backspace,
                    _ => Continue,
                };
            }
            Continue
        })
        .unwrap_or(Continue)
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
}

fn clip_if_needed(text: String, max_length: usize) -> String {
    if text.len() > max_length {
        return format!("{}...", &text[0..max_length - 3]);
    }
    text.to_string()
}

fn date_to_display(date: String) -> String {
    let now = Utc::now();
    NaiveDateTime::parse_from_str(&*date, "%Y-%m-%d %H:%M:%S")
        .map(|parsed_datetime| parsed_datetime.and_utc())
        .map(|datetime_utc| now.signed_duration_since(datetime_utc))
        .map(|duration| {
            if duration.num_weeks() > 0 {
                return format_time(duration.num_weeks(), "weeks".to_string());
            }
            if duration.num_days() > 0 {
                return format_time(duration.num_days(), "days".to_string());
            }
            if duration.num_hours() > 0 {
                return format_time(duration.num_hours(), "hours".to_string());
            }
            if duration.num_minutes() > 0 {
                return format_time(duration.num_minutes(), "minutes".to_string());
            }
            if duration.num_seconds() > 0 {
                return format_time(duration.num_seconds(), "seconds".to_string());
            }
            "Date could not be displayed".to_string()
        })
        .unwrap_or_else(|_| "Date could not be displayed".to_string())
}

fn format_time(amount: i64, time_measurement: String) -> String {
    format!("{} {} ago", amount, time_measurement)
}
