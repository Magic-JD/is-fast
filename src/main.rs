use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::process::{Command, Stdio};
use std::io::{Stdout, Write};
use std::io::{stdout, Result};
use crossterm::{
    event::{self, KeyCode, KeyEvent},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    widgets::{Block, Borders, Paragraph},
    layout::{Layout, Constraint, Direction},
    style::{Style, Color},
};

fn main() -> Result<()> {
    // Enable raw mode (no echo, captures key events)
    enable_raw_mode()?;
    let mut out = stdout();
    execute!(out, EnterAlternateScreen)?; // Use alternate screen buffer
    let backend = CrosstermBackend::new(out);
    let mut terminal = Terminal::new(backend)?;

    // Read user input from command line args
    let message: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let links = get_top_links(message);
    let mut index = 0;
    let link = links.get(index);
    let mut page = show_link(link);
    draw_page(&mut terminal, &mut page, link)?;
    loop {

        // Handle user input
        if let event::Event::Key(KeyEvent { code, .. }) = event::read()? {
            match code {
                KeyCode::Char('q') => break, // Quit
                KeyCode::Char('n') if index < links.len() - 1 => {
                    index += 1;
                    let link = links.get(index);
                    page = show_link(link);
                    draw_page(&mut terminal, &mut page, link)?;
                }
                KeyCode::Char('b') if index > 0 => {
                    index -= 1;
                    let link = links.get(index);
                    page = show_link(link);
                    draw_page(&mut terminal, &mut page, link)?;
                }
                _ => {}
            }
        }
    }

    // Cleanup
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen)?;
    Ok(())
}

fn draw_page(terminal: &mut Terminal<CrosstermBackend<Stdout>>, mut page: &mut String, link: Option<&Link>) -> Result<()> {
    terminal.draw(|frame| {
        let size = frame.size();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(100)].as_ref());

        let area = layout.split(size)[0];

        let block = Block::default()
            .title(link.map(|link| link.title.clone()).unwrap_or_else(|| "No Title".to_string()))
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Green));

        let paragraph = Paragraph::new(page.clone())
            .style(Style::default().fg(Color::White))
            .block(block);

        frame.render_widget(paragraph, area);
    })?;
    Ok(())
}

fn show_link(links: Option<&Link>) -> String {
    let first_url = links.map(|link| link.url.clone()).ok_or("No links available").unwrap();
    fetch_url(first_url)
}

fn fetch_url(first_url: String) -> String {
    let client = Client::new();
    let res = client
        .get(format!("https://{}", first_url))
        .send().unwrap()
        .error_for_status().unwrap()
        .text().unwrap();
    let selector = Selector::parse("h1, p, br").unwrap();
    let page: String = Html::parse_document(&res)
        .select(&selector)
        .map(|ele| ele.text().collect::<Vec<_>>().join("\n"))
        .collect();

    let columns = std::env::var("COLUMNS").unwrap_or_else(|_| "180".to_string());
    let cols = columns.parse::<usize>().unwrap_or(180).saturating_sub(5);
    let output = Command::new("pandoc")
        .arg("-f").arg("html")
        .arg("-t").arg("ansi")
        .arg("--columns").arg(cols.to_string())
        .env("TERM", "xterm-256color")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            if let Some(stdin) = child.stdin.as_mut() {
                stdin.write_all(page.as_bytes())?;
            }
            child.wait_with_output()
        });
    match output {
        Ok(output) if output.status.success() => {
            let ansi_text = String::from_utf8_lossy(&output.stdout);
            format!("{}", ansi_text)
        }
        Ok(output) => {
            format!("Error: {:?}", String::from_utf8_lossy(&output.stderr))
        }
        Err(e) => {
            format!("Failed to run pandoc: {}", e)
        }
    }
}

fn get_top_links(message: String) -> Vec<Link> {
    let url = format!("https://html.duckduckgo.com/html/?q={}", message);
    let client = Client::new();
    let html = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .send()
        .expect("Failed to fetch search results")
        .text()
        .expect("Failed to read response");
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();
    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(element_title, element_url)| {
            let title = element_title.text().collect::<Vec<_>>().join(" ").trim().to_owned();
            let url = element_url.text().collect::<Vec<_>>().join(" ").trim().to_owned();
            Link { title, url }
        })
        .collect::<Vec<_>>()
}

struct Link {
    url: String,
    title: String,
}
