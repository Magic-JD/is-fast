use crate::actions::direct;
use crate::database::connect::{remove_history, HistoryData};
use crate::tui::display::Display;
use crate::tui::history::Action::{Continue, Down, Exit, Open, Up};
use chrono::{NaiveDateTime, Utc};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Constraint;
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Row, Table, TableState};
use Action::Delete;

const INSTRUCTIONS: &'static str = " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑ | Open: ↵ | Delete: d";

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
        history.reverse();
        let mut state = TableState::default();
        state.select(Some(history.len().saturating_sub(1)));
        let mut rows = create_rows(history.clone());
        let mut table = create_table(&mut rows);
        self.display
            .draw_table(&table, history.len() as u16, "History".to_string(), &mut state)
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
                    history.into_iter()
                        .collect::<Vec<_>>()
                        .get(idx)
                        .map(HistoryData::clone)
                        .inspect(|history_data| {
                            direct::run(Some(history_data.title.clone()), history_data.url.clone());
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
                            );
                        }
                    }
                }
                Delete => {
                    let ref_state = &mut state;
                    let removed = history.remove(ref_state.selected().unwrap_or_else(|| 0));
                    _ = remove_history(&removed.url);
                    table = create_table(&mut create_rows(history.clone()));
                    self.display
                        .draw_table(&table, history.len() as u16, "History".to_string(), &mut state)
                        .expect("TODO: panic message");
                }
            }
        }
    }
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

fn create_rows(history: Vec<HistoryData>) -> Vec<Row<'static>> {
    let rows: Vec<Row> = history
        .iter()
        .map(|h| {
            let cells = vec![
                Cell::from(clip_if_needed(h.title.clone(), 100))
                    .style(Style::default().fg(Color::Red)),
                Cell::from(clip_if_needed(h.url.clone(), 60))
                    .style(Style::default().fg(Color::Green)),
                Cell::from(date_to_display(h.time.clone()))
                    .style(Style::default().fg(Color::Cyan)),
            ];
            Row::new(cells)
        })
        .collect();
    rows
}

fn handle_input() -> Action {
    event::read()
        .map(|event| {
            if let event::Event::Key(KeyEvent { code, .. }) = event {
                return match code {
                    KeyCode::Char('q') => Exit,
                    KeyCode::Up | KeyCode::Char('k') => Up,
                    KeyCode::Down | KeyCode::Char('j') => Down,
                    KeyCode::Enter => Open,
                    KeyCode::Char('d') => Delete,
                    _ => { Continue }
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
