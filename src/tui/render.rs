use crate::links::link::Link;
use crate::tui::events;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use once_cell::sync::Lazy;
use ratatui::prelude::Text;
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
use std::io::{stdout, Result, Stdout};
use std::sync::Mutex;

const INSTRUCTIONS: &'static str = " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o";
const TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(|| Style::default().fg(Color::Green));
const TERMINAL: Lazy<Mutex<Terminal<CrosstermBackend<Stdout>>>> = Lazy::new(|| Mutex::new(startup()));

pub fn loading() -> Result<()> {
    draw(&Paragraph::default(),
        " Loading...".to_string(),
        0,
    )
}

pub fn show(links: &Vec<Link>) {
    if links.is_empty() {
        eprintln!("No results found");
    }
    let mut index = 0;
    let mut page = links
        .get(index)
        .map(|link| link.get_content())
        .unwrap_or_else(|| Paragraph::new(Text::from(String::from("Index out of bounds"))));
    let mut scroll_offset = 0;
    results_page(&page, links.get(index), scroll_offset)
        .unwrap_or_else(|err| shutdown_with_error(&err.to_string()));
    let binding = TERMINAL;
    let mut terminal = binding.lock().unwrap();
    let height = terminal.get_frame().area().height;
    loop {
        if events::handle_input(
            &mut index,
            &links,
            &mut page,
            &mut scroll_offset,
            height - 5,
        )
        .map_err(|e| {
            eprintln!("Error: {}", e);
            true
        })
        .unwrap_or(true)
        {
            break;
        }
    }
    shutdown();
}

pub fn results_page(
    page: &Paragraph,
    link: Option<&Link>,
    scroll_offset: u16,
) -> Result<()> {
    let title = link
        .map(|l| format!(" {} ({}) ", l.title, l.url))
        .unwrap_or("No Title".to_string());
    draw(page, title, scroll_offset)
}

fn draw(
    page: &Paragraph,
    title: String,
    scroll_offset: u16,
) -> Result<()> {
    let binding = TERMINAL;
    let mut terminal = binding.lock().unwrap();
    terminal.clear()?;
    terminal.draw(|frame| {
        let size = frame.area();
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());
        let area = layout.split(size)[0];
        let block = Block::default()
            .title(tui_border_span(title.as_str()))
            .title_bottom(tui_border_span(INSTRUCTIONS))
            .borders(Borders::TOP)
            .style(TUI_BORDER_COLOR.clone());
        let paragraph = Paragraph::from(page.clone())
            .block(block)
            .style(Style::default().fg(Color::White))
            .wrap(Wrap { trim: false })
            .scroll((scroll_offset, 0));

        frame.render_widget(paragraph, area);
    })?;
    Ok(())
}

fn tui_border_span(text: &str) -> Span {
    Span::styled(text, TUI_BORDER_COLOR.clone().add_modifier(Modifier::BOLD))
}

pub fn startup() -> Terminal<CrosstermBackend<Stdout>> {
    // This can panic if startup not handled properly.
    enable_raw_mode().unwrap();
    let mut out = stdout();
    execute!(out, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(out);
    Terminal::new(backend).unwrap()
}

pub fn shutdown_with_error(error: &str) -> ! {
    shutdown();
    eprintln!("{}", error);
    std::process::exit(1);
}

fn shutdown() {
    // This can panic if shutdown cannot be handled properly.
    let binding = TERMINAL;
    let mut terminal = binding.lock().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
    disable_raw_mode().unwrap();
}
