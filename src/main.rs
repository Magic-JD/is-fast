mod config;
mod error;
mod extract_formatted;
mod extract_links;
mod input;
mod link;
mod scrape;
mod syntax_highlighting;
mod ui;
mod cli;

use crate::cli::Cli;
use crate::config::generate_config;
use crate::extract_links::extract_links;
use crate::scrape::scrape;
use clap::{Parser};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Stdout};

fn main() {
    let args = Cli::parse();
    if args.generate_config {
        generate_config();
        return;
    }
    let search_term = args.query.map(|query| query.join(" "));
    if let Some(search_term) = search_term {
        run_search(search_term);
        return;
    }
    eprintln!("No search term provided!");
}

fn run_search(search_term: String) {
    let mut terminal = startup();
    let mut index = 0;
    ui::draw_loading(&mut terminal)
        .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let links = &scrape(&format!(
        "https://html.duckduckgo.com/html/?q={}",
        &search_term
    ))
    .map(|html| extract_links(&html))
    .unwrap_or_else(|err| shutdown_with_error(&mut terminal, &err.to_string()));
    let mut page = links
        .get(index)
        .map(|link| link.get_content())
        .unwrap_or_else(|| Paragraph::new(Text::from(String::from("Index out of bounds"))));
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
