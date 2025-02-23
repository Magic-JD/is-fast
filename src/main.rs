use ncurses::*;
use reqwest::blocking::Client;
use scraper::{Html, Selector};

fn main() {
    setup_tui();
    let mut index = 0;
    let message: String = std::env::args().skip(1).collect::<Vec<String>>().join(" ");
    let links = get_top_links(message);
    let mut page = show_link(links.get(index));

    mvprintw(1, 1, format!("{}", page).as_str());
    refresh();
    loop {
        let ch = getch();
        if ch == 'q' as i32 {
            break;
        }
        if ch == 'n' as i32 && index < 4 {
            index +=1;
            page = show_link(links.get(index));
            mvprintw(1, 1, format!("{}", page).as_str());
            refresh();
        }
        if ch == 'b' as i32 && index > 0 {
            index -=1;
            page = show_link(links.get(index));
            mvprintw(1, 1, format!("{}", page).as_str());
            refresh();
        }
    }
    endwin();
}

fn show_link(links: Option<&Link>) -> String {
    let first_url = links.map(|link| link.url.clone()).ok_or("No links available").unwrap();
    fetch_url(first_url).unwrap()
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

struct Link {
    url: String,
    title: String,
}
