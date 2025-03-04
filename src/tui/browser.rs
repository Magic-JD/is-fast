use crate::errors::error::IsError;
use crate::links::cache::new_page;
use crate::links::link::Link;
use crate::tui::browser::Action::{Down, Exit, Next, Open, PageDown, PageUp, Previous, Up};
use crate::tui::display::Display;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::widgets::Paragraph;

const INSTRUCTIONS: &str = " Quit: q/Esc | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o";

pub struct Browser {
    display: Display,
}

impl Browser {
    pub fn new() -> Self {
        let display = Display::new(INSTRUCTIONS.to_string());
        display.loading().unwrap();
        Browser { display }
    }

    pub fn browse(mut self, links: Vec<Link>, history_active: bool) {
        let height = self.display.height();
        if links.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        let mut page = new_page(&index, &links, history_active);
        let mut scroll_offset = 0;
        self.results_page(&page, links.get(index), scroll_offset)
            .unwrap_or_else(|err| self.display.shutdown_with_error(&err.to_string()));
        loop {
            match handle_input() {
                Exit => break,
                Next => {
                    index = (index + 1).min(links.len().saturating_sub(1));
                    self.change_page(
                        &index,
                        &links,
                        &mut page,
                        &mut scroll_offset,
                        history_active,
                    )
                    .unwrap();
                }
                Previous => {
                    index = index.saturating_sub(1);
                    self.change_page(
                        &index,
                        &links,
                        &mut page,
                        &mut scroll_offset,
                        history_active,
                    )
                    .unwrap();
                }
                Down => {
                    scroll_offset += 1;
                    self.draw(&index, &links, &page, &scroll_offset).unwrap();
                }
                Up => {
                    scroll_offset = scroll_offset.saturating_sub(1);
                    self.draw(&index, &links, &page, &scroll_offset).unwrap();
                }
                PageUp => {
                    scroll_offset = scroll_offset.saturating_sub(height / 2);
                    self.draw(&index, &links, &page, &scroll_offset).unwrap();
                }
                PageDown => {
                    scroll_offset += height / 2;
                    self.draw(&index, &links, &page, &scroll_offset).unwrap();
                }
                Open => {
                    open_link(&index, &links);
                }
                Action::Continue => {}
            }
        }
        self.display.shutdown();
    }

    pub fn results_page(
        &self,
        page: &Paragraph,
        link: Option<&Link>,
        scroll_offset: u16,
    ) -> std::io::Result<()> {
        let title = link
            .map(|l| format!(" {} ({}) ", l.title, l.url))
            .unwrap_or("No Title".to_string());
        self.display.draw(page, title, scroll_offset)
    }

    fn change_page(
        &self,
        index: &usize,
        links: &[Link],
        page: &mut Paragraph,
        scroll_offset: &mut u16,
        history_active: bool,
    ) -> Result<(), IsError> {
        self.display.loading()?;
        *page = new_page(index, links, history_active);
        *scroll_offset = 0;
        self.draw(index, links, page, scroll_offset)?;
        Ok(())
    }

    fn draw(
        &self,
        index: &usize,
        links: &[Link],
        page: &Paragraph,
        scroll_offset: &u16,
    ) -> Result<(), IsError> {
        self.results_page(page, links.get(*index), *scroll_offset)
            .map_err(|e| IsError::General(e.to_string()))?;
        Ok(())
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
        .map(|link| format!("https://{}", link.url))
        .and_then(|url| open::that(&url).err())
        .iter()
        .for_each(|e| println!("{}", e));
}
