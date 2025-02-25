use crate::models::Link;
use ratatui::style::Modifier;
use ratatui::text::Span;
use ratatui::widgets::Wrap;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use std::io::{Error, Result, Stdout};

pub fn draw_loading(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    draw(terminal, &Paragraph::default(), " Loading...".to_string(), 0)
}

pub fn draw_page(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    page: &Paragraph,
    link: Option<&Link>,
    scroll_offset: u16,
) -> Result<()> {
    let title = link.map(|l| format!(" {} ({}) ", l.title, l.url)).unwrap_or("No Title".to_string());
    draw(terminal, page, title, scroll_offset)
}

fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, page: &Paragraph, title: String, scroll_offset: u16) -> Result<()> {
    terminal.clear()?;
    terminal.draw(|frame| {
        let size = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());
        let area = layout.split(size)[0];

        let block = Block::default()
            .title(
                Span::styled(
                    String::from(title),
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            ).title_bottom(
            Span::styled(
                " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← ",
                Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
            ), )
            .borders(Borders::TOP)
            .style(Style::default().fg(Color::Green));
        let paragraph = Paragraph::from(page.clone())
            .block(block)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset, 0));

        frame.render_widget(paragraph, area);
    })?;
    Ok(())
}

