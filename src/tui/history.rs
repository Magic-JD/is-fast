use crate::database::connect::HistoryData;
use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use crate::tui::browser::Browser;
use crate::tui::display::Display;
use crate::tui::history::Action::{Continue, Exit, Open};
use chrono::{NaiveDateTime, Utc};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::layout::Constraint;
use ratatui::prelude::Modifier;
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Row, Table, TableState};

const INSTRUCTIONS: &'static str = " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑";

pub struct History {
    display: Display,
}

impl History {
    pub fn new() -> Self {
        History {
            display: Display::new(INSTRUCTIONS.to_string()),
        }
    }

    pub fn show_history(mut self, history: Vec<HistoryData>) {
        if history.is_empty() {
            self.display.shutdown();
            eprintln!("No history found");
            return;
        }
        let mut state = TableState::default();
        state.select(Some(history.len().saturating_sub(1)));
        let row_count = history.len() as u16;
        let rows: Vec<Row> = history
            .iter()
            .map(|h| {
                let cells = vec![
                    Cell::from(h.index.to_string()).style(Style::default().fg(Color::Cyan)),
                    Cell::from(date_to_display(h.time.clone()))
                        .style(Style::default().fg(Color::Cyan)),
                    Cell::from(clip_if_needed(h.title.clone(), 100))
                        .style(Style::default().fg(Color::Red)),
                    Cell::from(clip_if_needed(h.url.clone(), 60))
                        .style(Style::default().fg(Color::Green)),
                ];
                Row::new(cells)
            })
            .rev()
            .collect();

        let table = Table::from_iter(rows)
            .widths(&[
                Constraint::Percentage(3),
                Constraint::Percentage(15),
                Constraint::Percentage(40),
                Constraint::Percentage(40),
            ])
            .column_spacing(1)
            .highlight_symbol(">> ")
            .row_highlight_style(Style::default().add_modifier(Modifier::BOLD));
        self.display
            .draw_table(&table, row_count, "History".to_string(), &mut state)
            .expect("TODO: panic message");
        loop {
            match self.handle_input(&mut state, history.len(), &table) {
                Continue => {}
                Exit => {
                    self.display.shutdown();
                    break;
                }
                Open(idx) => {
                    self.display.shutdown();
                    let rev_history = history.into_iter().rev().collect::<Vec<_>>();
                    _ = rev_history
                        .get(idx)
                        .map(HistoryData::clone)
                        .map(|history_data| {
                            Link::new(
                                history_data.title.clone(),
                                history_data.url.clone(),
                                move || {
                                    scrape(
                                        &format!("https://{}", history_data.url.clone())
                                            .to_string(),
                                    )
                                },
                            )
                        })
                        .map(|link| Browser::new().browse(vec![link]));
                    break;
                }
            }
        }
    }

    fn handle_input(&self, state: &mut TableState, max: usize, table: &Table) -> Action {
        event::read()
            .map(|event| {
                if let event::Event::Key(KeyEvent { code, .. }) = event {
                    return match code {
                        KeyCode::Char('q') => Exit,
                        KeyCode::Up | KeyCode::Char('k') => {
                            if let Some(selected) = state.selected() {
                                if selected > 0 {
                                    state.select(Some(selected - 1));
                                    _ = self.display.draw_table(
                                        table,
                                        max as u16,
                                        "History".to_string(),
                                        state,
                                    );
                                }
                            }
                            Continue
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if let Some(selected) = state.selected() {
                                if selected < max - 1 {
                                    state.select(Some(selected + 1));
                                    _ = self.display.draw_table(
                                        table,
                                        max as u16,
                                        "History".to_string(),
                                        state,
                                    );
                                }
                            }
                            Continue
                        }
                        KeyCode::Enter => state
                            .selected()
                            .map(|idx| Open(idx))
                            .unwrap_or_else(|| Continue),
                        _ => Continue,
                    };
                }
                Continue
            })
            .unwrap_or(Continue)
    }
}

enum Action {
    Exit,
    Continue,
    Open(usize),
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
