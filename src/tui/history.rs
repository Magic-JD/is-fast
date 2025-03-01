use crate::database::connect::HistoryData;
use crate::errors::error::MyError;
use crate::tui::display::Display;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::style::{Color, Style};
use ratatui::widgets::{Cell, Row, Table};

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
        let rows: Vec<Row> = history
            .iter()
            .map(|h| {
                let cells = vec![
                    Cell::from(h.index.to_string()),
                    Cell::from(h.time.clone()),
                    Cell::from(h.title.clone()),
                    Cell::from(h.url.clone()),
                ];
                Row::new(cells)
            })
            .collect();

        let table = Table::from_iter(rows)
            .header(
                Row::new(vec!["Index", "Time", "Title", "URL"])
                    .style(Style::default().fg(Color::Yellow)),
            )
            .widths(&[
                ratatui::layout::Constraint::Percentage(10),
                ratatui::layout::Constraint::Percentage(20),
                ratatui::layout::Constraint::Percentage(40),
                ratatui::layout::Constraint::Percentage(30),
            ]);
        self.display
            .draw_table(&table, "History".to_string(), 0)
            .expect("TODO: panic message");
        loop {
            if self
                .handle_input()
                .map_err(|e| {
                    eprintln!("Error: {}", e);
                    true
                })
                .unwrap_or(true)
            {
                break;
            }
        }
        self.display.shutdown();
    }

    fn handle_input(&self) -> Result<bool, MyError> {
        if let event::Event::Key(KeyEvent {
            code, modifiers, ..
        }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => return Ok(true),
                _ => {}
            }
        }
        Ok(false)
    }
}
