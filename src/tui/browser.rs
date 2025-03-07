use crate::database::connect::add_history;
use crate::search::link::Link;
use crate::search::scrape::format_url;
use crate::transform::cache::{get_content, preload};
use crate::transform::page::PageExtractor;
use crate::tui::browser::Action::{Down, Exit, Next, Open, PageDown, PageUp, Previous, Up};
use crate::tui::display::{default_block, Display, TUI_BORDER_COLOR};
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Alignment;
use ratatui::prelude::{Color, Line, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};

static PAGE_INSTRUCTIONS: &str = " Quit: q/Esc | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o ";

pub struct Browser {
    display: Display,
}

impl Browser {
    pub fn new() -> Self {
        let mut display = Display::new();
        display.loading();
        Browser { display }
    }

    pub fn shutdown(&mut self) {
        self.display.shutdown();
    }

    pub fn browse(mut self, links: &[Link], extractor: &PageExtractor, history_active: bool) {
        let height = self.display.height();
        let mut scroll: u16 = 0;
        if links.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        let (mut title, mut page) = new_page(index, links, extractor, history_active);
        let mut border = default_block(&title, PAGE_INSTRUCTIONS);
        let mut page_numbers = draw_page_numbers(index + 1, links.len());
        self.display.draw_page(&page, &border, &page_numbers);
        loop {
            match handle_input() {
                Exit => break,
                Next => {
                    if index < links.len() - 1 {
                        scroll = 0;
                        index += 1;
                        self.display.loading();
                        (title, page) = new_page(index, links, extractor, history_active);
                        border = default_block(&title, PAGE_INSTRUCTIONS);
                        page_numbers = draw_page_numbers(index + 1, links.len());
                        self.display.draw_page(&page, &border, &page_numbers);
                    }
                }
                Previous => {
                    if index > 0 {
                        scroll = 0;
                        index -= 1;
                        self.display.loading();
                        (title, page) = new_page(index, links, extractor, history_active);
                        border = default_block(&title, PAGE_INSTRUCTIONS);
                        page_numbers = draw_page_numbers(index + 1, links.len());
                        self.display.draw_page(&page, &border, &page_numbers);
                    }
                }
                Down => {
                    scroll = scroll.saturating_add(1);
                    page = page.scroll((scroll, 0));
                    self.display.draw_page(&page, &border, &page_numbers);
                }
                Up => {
                    scroll = scroll.saturating_sub(1);
                    page = page.scroll((scroll, 0));
                    self.display.draw_page(&page, &border, &page_numbers);
                }
                PageUp => {
                    scroll = scroll.saturating_sub(height / 2);
                    page = page.scroll((scroll, 0));
                    self.display.draw_page(&page, &border, &page_numbers);
                }
                PageDown => {
                    scroll = scroll.saturating_add(height / 2);
                    page = page.scroll((scroll, 0));
                    self.display.draw_page(&page, &border, &page_numbers);
                }
                Open => {
                    open_link(index, links);
                }
                Action::Continue => {}
            }
        }
        self.display.shutdown();
    }
}

fn handle_input() -> Action {
    if let Ok(event::Event::Key(KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        ..
    })) = event::read()
    {
        return match code {
            KeyCode::Char('q') | KeyCode::Esc => Exit,
            KeyCode::Char('n') | KeyCode::Right => Next,
            KeyCode::Char('b') | KeyCode::Left => Previous,
            KeyCode::Down | KeyCode::Char('j') => Down,
            KeyCode::Up | KeyCode::Char('k') => Up,
            KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => PageUp,
            KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => PageDown,
            KeyCode::Char('o') => Open,
            _ => Action::Continue,
        };
    }
    Action::Continue
}

enum Action {
    Exit,
    Open,
    Up,
    Down,
    PageUp,
    PageDown,
    Next,
    Previous,
    Continue,
}

fn open_link(index: usize, links: &[Link]) {
    links
        .get(index)
        .map(|link| format_url(&link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{e}"));
}

pub fn new_page(
    index: usize,
    links: &[Link],
    extractor: &PageExtractor,
    history_active: bool,
) -> (String, Paragraph<'static>) {
    if let Some(link) = links.get(index + 1) {
        preload(link, extractor); // Initiate the call to get the page after this one
    }
    links
        .get(index)
        .inspect(|link| {
            if history_active {
                _ = add_history(link);
            }
        })
        .map(|link| (link, get_content(link, extractor)))
        .map_or_else(
            || {
                (
                    String::from("None"),
                    Paragraph::new(Text::from(String::from("Index out of bounds"))),
                )
            },
            |(link, paragraph)| {
                (
                    extract_title(link),
                    paragraph
                        .style(Style::default().fg(Color::White))
                        .wrap(Wrap { trim: false })
                        .scroll((0, 0)),
                )
            },
        )
}
fn extract_title(link: &Link) -> String {
    format!(" {} ({}) ", link.title, link.url)
}

pub fn draw_page_numbers(index: usize, pages: usize) -> Text<'static> {
    Text::from(Line::styled(
        format!(" [{index}/{pages}] "),
        *TUI_BORDER_COLOR,
    ))
    .alignment(Alignment::Right)
}
