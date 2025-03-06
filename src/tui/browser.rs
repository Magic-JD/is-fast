use crate::database::connect::add_history;
use crate::errors::error::IsError;
use crate::search::link::Link;
use crate::search::scrape::format_url;
use crate::transform::cache::{get_content, preload};
use crate::transform::page::PageExtractor;
use crate::tui::browser::Action::{Down, Exit, Next, Open, PageDown, PageUp, Previous, Up};
use crate::tui::display::Display;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::prelude::{Color, Style, Text};
use ratatui::widgets::{Paragraph, Wrap};

pub struct Browser {
    display: Display,
}

impl Browser {
    pub fn new() -> Self {
        let display = Display::new();
        display.loading().unwrap();
        Browser { display }
    }

    pub fn shutdown(&mut self) {
        self.display.shutdown()
    }

    pub fn browse(mut self, links: Vec<Link>, extractor: PageExtractor, history_active: bool) {
        let height = self.display.height();
        let mut scroll: u16 = 0;
        if links.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        let (mut title, mut page) = new_page(&index, &links, &extractor, history_active);
        self.display
            .draw_page(&page, &title, index + 1, links.len())
            .unwrap_or_else(|err| self.display.shutdown_with_error(&err.to_string()));
        loop {
            match handle_input() {
                Exit => break,
                Next => {
                    if index < links.len() - 1 {
                        scroll = 0;
                        index += 1;
                        self.change_page(
                            &index,
                            &links,
                            &mut page,
                            &mut title,
                            &extractor,
                            history_active,
                        )
                        .unwrap();
                    }
                }
                Previous => {
                    if index > 0 {
                        scroll = 0;
                        index -= 1;
                        self.change_page(
                            &index,
                            &links,
                            &mut page,
                            &mut title,
                            &extractor,
                            history_active,
                        )
                        .unwrap();
                    }
                }
                Down => {
                    scroll = scroll.saturating_add(1);
                    page = page.scroll((scroll, 0));
                    let _ = self
                        .display
                        .draw_page(&page, &title, index + 1, links.len());
                }
                Up => {
                    scroll = scroll.saturating_sub(1);
                    page = page.scroll((scroll, 0));
                    let _ = self
                        .display
                        .draw_page(&page, &title, index + 1, links.len());
                }
                PageUp => {
                    scroll = scroll.saturating_sub(height / 2);
                    page = page.scroll((scroll, 0));
                    let _ = self
                        .display
                        .draw_page(&page, &title, index + 1, links.len());
                }
                PageDown => {
                    scroll = scroll.saturating_add(height / 2);
                    page = page.scroll((scroll, 0));
                    let _ = self
                        .display
                        .draw_page(&page, &title, index + 1, links.len());
                }
                Open => {
                    open_link(&index, &links);
                }
                Action::Continue => {}
            }
        }
        self.display.shutdown();
    }

    fn change_page(
        &self,
        index: &usize,
        links: &[Link],
        page: &mut Paragraph,
        title: &mut String,
        extractor: &PageExtractor,
        history_active: bool,
    ) -> Result<(), IsError> {
        self.display.loading()?;
        let (t, p) = new_page(index, links, extractor, history_active);
        *title = t;
        *page = p;
        self.display
            .draw_page(page, title, index + 1, links.len())
            .map_err(|e| IsError::General(e.to_string()))
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

fn open_link(index: &usize, links: &[Link]) {
    links
        .get(*index)
        .map(|link| format_url(&link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{}", e));
}

pub fn new_page(
    index: &usize,
    links: &[Link],
    extractor: &PageExtractor,
    history_active: bool,
) -> (String, Paragraph<'static>) {
    if let Some(link) = links.get(*index + 1) {
        preload(link, extractor); // Initiate the call to get the page after this one
    }
    links
        .get(*index)
        .inspect(|link| {
            if history_active {
                _ = add_history(link)
            }
        })
        .map(|link| (link, get_content(link, extractor)))
        .map(|(link, paragraph)| {
            (
                extract_title(link),
                paragraph
                    .style(Style::default().fg(Color::White))
                    .wrap(Wrap { trim: false })
                    .scroll((0, 0)),
            )
        })
        .unwrap_or_else(|| {
            (
                String::from("None"),
                Paragraph::new(Text::from(String::from("Index out of bounds"))),
            )
        })
}
fn extract_title(link: &Link) -> String {
    format!(" {} ({}) ", link.title, link.url)
}
