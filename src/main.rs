use ncurses::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::env;
use std::fs::write;
use std::process::Command;
use tempfile::NamedTempFile;

fn main() {
   // setup_tui();
    let message: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let links = get_top_links(message);
    showLink(links.get(0));

    loop {
        let ch = getch();
        if ch == 'q' as i32 {
            break;
        }
        if ch == 'n' as i32 {
            showLink(links.get(1));
        }
    }
}

fn showLink(links: Option<&Link>) {
    let first_url = links.map(|link| link.url.clone()).ok_or("No links available").unwrap();
    let page = fetch_url(first_url).unwrap();

    // Write to a temporary file
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    write(temp_file.path(), &page).expect("Failed to write to temp file");

    // Run bat for pagination
    Command::new("bat")
        .arg(temp_file.path()) // Pass temp file to bat
        .arg("--paging=always") // Ensure pagination
        .status()
        .expect("Failed to execute bat");
}

fn fetch_url(first_url: String) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let res = client
        .get(format!("https://{}", first_url))
        .send()?
        .error_for_status()?
        .text()?;
    let selector = Selector::parse("body").unwrap();
    let page = Html::parse_document(&res).select(&selector).map(|ele| ele.text().collect::<Vec<_>>().join(" ")).collect();
    Ok(page)
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

fn setup_tui() {
    initscr();
    raw();
    keypad(stdscr(), true);
    noecho();
}

fn debug_out(debug: String) -> std::io::Result<()> {
    let file_path = Path::new("result.html"); // File in the project directory
    let mut file = File::create(&file_path)?; // Create (or overwrite) the file

    file.write_all(debug.as_bytes())?; // Write content to the file

    println!("File written to {:?}", file_path);
    Ok(())
}

fn await_input() {
}

struct Link {
    url: String,
    title: String,
}
