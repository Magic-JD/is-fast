use crate::search::link::Link;
use crate::search::scrape::format_url;
use crate::transform::page::PageExtractor;
use crate::tui::browser::Action::{Down, Exit, Next, Open, PageDown, PageUp, Previous, Up};
use crate::tui::browser_widgets::{draw_page_numbers, new_page};
use crate::tui::display::Display;
use crate::tui::general_widgets::default_block;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

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
