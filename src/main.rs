use ncurses::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    setup_tui();
    let message: String = std::env::args()
        .skip(1)
        .collect::<Vec<String>>()
        .join(" ");
    let client = Client::new();
    
    let url = format!("https://html.duckduckgo.com/html/?q={}", message);

    let html = client.get(&url)
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
        .send()
        .expect("Failed to fetch search results")
        .text()
        .expect("Failed to read response");
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();
    let mut results = String::new();
    for (elementTitle, elementUrl) in document.select(&selector_title).zip(document.select(&selector_url)).take(5) {
        let title = elementTitle.text().collect::<Vec<_>>().join(" ");
        let url = elementUrl.text().collect::<Vec<_>>().join(" ");

        let result = format!("\nTitle: {}\nLink: {}\n\n", title, url);
        results.push_str(&result);
    }
    mvprintw(1, 1, &results);
    refresh();
    await_input();
    endwin();
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

fn await_input(){
    loop {
        let ch = getch();
        if ch == 'q' as i32 {
            break;
        }
    }
}
