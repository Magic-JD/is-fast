use crate::models::Link;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use ratatui::text::{Line, Text, Span};
use ratatui::widgets::Wrap;
use std::io::{Result, Stdout};
use ansi_to_tui::IntoText;
use ratatui::layout::Alignment;

pub fn draw_loading(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());
        let area = layout.split(size)[0];

        let block = Block::default()
            .title("Loading...")
            .borders(Borders::TOP)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(block, area);
    })?;
    Ok(())
}

pub fn draw_page(terminal: &mut Terminal<CrosstermBackend<Stdout>>, page: &str, link: Option<&Link>, scroll_offset: u16) -> Result<()> {
    terminal.clear()?;
    terminal.draw(|frame| {
        let size = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());

        let area = layout.split(size)[0];

        let block = Block::default()
            .title(link.map(|l| l.title.clone()).unwrap_or("No Title".to_string()))
            .borders(Borders::TOP)
            .style(Style::default().fg(Color::Green));

        let text = page.into_text().unwrap();
        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true })
            .scroll((scroll_offset, 0));

        frame.render_widget(paragraph, area);
    })?;
    Ok(())
}
