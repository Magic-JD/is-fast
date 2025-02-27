use crate::links::link::Link;
use once_cell::sync::Lazy;
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
use std::io::{Result, Stdout};
use crate::formatting::format::UIComponent;

const INSTRUCTIONS: &'static str = " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o";
const TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(|| Style::default().fg(Color::Green));

pub fn loading(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    draw(terminal, &vec![], " Loading...".to_string(), 0)
}

pub fn page(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    page: &Vec<UIComponent>,
    link: Option<&Link>,
    scroll_offset: u16,
) -> Result<()> {
    let title = link.map(|l| format!(" {} ({}) ", l.title, l.url)).unwrap_or("No Title".to_string());
    draw(terminal, page, title, scroll_offset)
}


fn draw(terminal: &mut Terminal<CrosstermBackend<Stdout>>, page: &Vec<UIComponent>, title: String, scroll_offset: u16) -> Result<()> {
    terminal.clear()?;
    terminal.draw(|frame| {
        let size = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());
        let area = layout.split(size)[0];
        let block = Block::default()
            .title(tui_border_span(title.as_str())).title_bottom(
            tui_border_span(INSTRUCTIONS))
            .borders(Borders::TOP)
            .style(TUI_BORDER_COLOR.clone());
    })?;
    Ok(())
}

fn tui_border_span(text: &str) -> Span {
    Span::styled(text, TUI_BORDER_COLOR.clone().add_modifier(Modifier::BOLD))
}

