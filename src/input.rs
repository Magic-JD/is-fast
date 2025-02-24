use crossterm::event::{self, KeyCode, KeyEvent};
use std::io::Result;
use crate::{ui, fetch, models::Link};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::Stdout;

pub fn handle_input(
    index: &mut usize,
    links: &[Link],
    page: &mut String,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<bool> {
    if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Char('q') => return Ok(true), // Quit
            KeyCode::Char('n') if *index < links.len() - 1 => {
                *index += 1;
                *page = fetch::fetch_url(links.get(*index));
                ui::draw_page(terminal, page, links.get(*index))?;
            }
            KeyCode::Char('b') if *index > 0 => {
                *index -= 1;
                *page = fetch::fetch_url(links.get(*index));
                ui::draw_page(terminal, page, links.get(*index))?;
            }
            _ => {}
        }
    }
    Ok(false)
}
