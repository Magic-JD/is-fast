use std::io::{stdout, Stdout};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::CrosstermBackend;
use ratatui::prelude::Text;
use ratatui::Terminal;
use ratatui::widgets::Paragraph;
use crate::extract_links::extract_links;
use crate::scrape::scrape;
use crate::{input, ui};

pub fn run_search(search_term: String) {
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
