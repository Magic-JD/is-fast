mod ui;
mod input;
mod fetch;
mod models;
mod scrape;
mod extract;

use crossterm::{
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    execute,
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{stdout, Result};
use crate::extract::extract_links;
use crate::scrape::scrape;

fn main() -> Result<()> {
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    let message: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let links = extract_links(&scrape(&format!("https://html.duckduckgo.com/html/?q={}", &message)));
    let mut index = 0;
    let mut page = fetch::fetch_url(links.get(index));
    let mut scroll_offset = 0;
    ui::draw_page(&mut terminal, &page, links.get(index), scroll_offset)?;
    let height = terminal.get_frame().area().height;
    loop {
        if input::handle_input(&mut index, &links, &mut page, &mut terminal, &mut scroll_offset, height-5)? {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}
