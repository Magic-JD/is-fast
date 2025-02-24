use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};
use std::io::{Result, Stdout};
use crate::models::Link;

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

        let paragraph = Paragraph::new(page.to_string())
            .style(Style::default().fg(Color::White))
            .block(block)
            .scroll((scroll_offset, 0));

        frame.render_widget(paragraph, area);
    })?;
    Ok(())
}
