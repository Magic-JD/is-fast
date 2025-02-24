mod error;
mod extract;
mod fetch;
mod input;
mod models;
mod scrape;
mod ui;

use crate::extract::extract_links;
use crate::scrape::scrape;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};

fn main() {
    let mut terminal = startup();
    let mut index = 0;
    ui::draw_loading(&mut terminal)
        .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let message: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let links = &scrape(&format!("https://html.duckduckgo.com/html/?q={}", &message))
        .map(|html| extract_links(&html))
        .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let mut page = fetch::fetch_url(links.get(index))
        .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let mut scroll_offset = 0;
    ui::draw_page(&mut terminal, &page, links.get(index), scroll_offset)
        .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let height = terminal.get_frame().area().height;
    loop {
        if input::handle_input(
            &mut index,
            &links,
            &mut page,
            &mut terminal,
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
    shutdown(&mut terminal);
}

fn startup() -> Terminal<CrosstermBackend<Stdout>> {
    // This can panic if startup not handled properly.
    enable_raw_mode().unwrap();
    let mut out = stdout();
    execute!(out, EnterAlternateScreen).unwrap();
    let backend = CrosstermBackend::new(out);
    Terminal::new(backend).unwrap()
}

fn shutdown_with_error(terminal: &mut Terminal<CrosstermBackend<Stdout>>, error: &str) -> ! {
    shutdown(terminal);
    eprintln!("{}", error);
    std::process::exit(1);
}

fn shutdown(terminal: &mut Terminal<CrosstermBackend<Stdout>>) {
    // This can panic if shutdown cannot be handled properly.
    disable_raw_mode().unwrap();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
}
