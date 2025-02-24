use crate::error::MyError;
use crate::{models::Link, ui};
use crossterm::event::{self, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use std::result::Result;

pub fn handle_input(
    index: &mut usize,
    links: &[Link],
    page: &mut String,
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    scroll_offset: &mut u16,
    page_height: u16,
) -> Result<bool, MyError> {
    if let event::Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
        let total_lines = page.lines().count() as u16;
        let max_scroll = total_lines.saturating_sub(page_height);
        match code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Char('n') | KeyCode::Right if *index < links.len() - 1 => {
                ui::draw_loading(terminal)?;
                *index += 1;
                *page = links.get(*index).map(|link| link.get_content()).unwrap_or_else(|| String::from("Index out of bounds"));
                *scroll_offset = 0;
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('b') | KeyCode::Left if *index > 0 => {
                ui::draw_loading(terminal)?;
                *index -= 1;
                *page = links.get(*index).map(|link| link.get_content()).unwrap_or_else(|| String::from("Index out of bounds"));
                *scroll_offset = 0;
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                *scroll_offset = (*scroll_offset + 1).min(max_scroll);
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                *scroll_offset = scroll_offset.saturating_sub(1); // Scroll up
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = scroll_offset.saturating_sub(page_height);
                draw(index, links, page, terminal, scroll_offset)?;
            }
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                *scroll_offset = (*scroll_offset + page_height).min(max_scroll);
                draw(index, links, page, terminal, scroll_offset)?;
            }
            _ => {}
        }
    }
    Ok(false)
}

fn draw(index: &mut usize, links: &[Link], page: &mut String, terminal: &mut Terminal<CrosstermBackend<Stdout>>, scroll_offset: &mut u16) -> Result<(), MyError> {
    ui::draw_page(terminal, page, links.get(*index), *scroll_offset).map_err(|e| MyError::DisplayError(e.to_string()))?;
    Ok(())
}
