use crate::database::connect::add_history;
use crate::errors::error::MyError;
use crate::links::cache::{get_content, preload};
use crate::links::link::Link;
use crate::tui::display::Display;
use crossterm::event;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::prelude::Text;
use ratatui::widgets::Paragraph;

const INSTRUCTIONS: &'static str = " Quit: q | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o";

pub struct Browser {
    display: Display
}

impl Browser {
    pub fn new() -> Self {
        let display = Display::new(INSTRUCTIONS.to_string());
        display.loading().unwrap();
        Browser {
            display,
        }
    }

    pub fn browse(mut self, links: Vec<Link>){
        let height = self.display.height();
        if links.is_empty() {
            self.display.shutdown();
            eprintln!("No results found");
            return;
        }
        let mut index = 0;
        let mut page = self.new_page(&mut index, &links);
        let mut scroll_offset = 0;
        self.results_page(&page, links.get(index), scroll_offset)
            .unwrap_or_else(|err| self.display.shutdown_with_error(&err.to_string()));
        loop {
            if self.handle_input(
                &mut index,
                &links,
                &mut page,
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

    pub fn handle_input(
        &self,
        index: &mut usize,
        links: &[Link],
        page: &mut Paragraph<'static>,
        scroll_offset: &mut u16,
        page_height: u16,
    ) -> Result<bool, MyError> {
        if let event::Event::Key(KeyEvent {
                                     code, modifiers, ..
                                 }) = event::read()?
        {
            match code {
                KeyCode::Char('q') => return Ok(true),
                KeyCode::Char('n') | KeyCode::Right => {
                    *index = (*index + 1).min(links.len().saturating_sub(1));
                    self.change_page(index, links, page, scroll_offset)?;
                }
                KeyCode::Char('b') | KeyCode::Left if *index > 0 => {
                    *index = index.saturating_sub(1);
                    self.change_page(index, links, page, scroll_offset)?;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    *scroll_offset = *scroll_offset + 1;
                    self.draw(index, links, page, scroll_offset)?;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    *scroll_offset = scroll_offset.saturating_sub(1); // Scroll up
                    self.draw(index, links, page, scroll_offset)?;
                }
                KeyCode::Char('u') if modifiers.contains(KeyModifiers::CONTROL) => {
                    *scroll_offset = scroll_offset.saturating_sub(page_height / 2);
                    self.draw(index, links, page, scroll_offset)?;
                }
                KeyCode::Char('d') if modifiers.contains(KeyModifiers::CONTROL) => {
                    *scroll_offset = *scroll_offset + (page_height / 2);
                    self.draw(index, links, page, scroll_offset)?;
                }
                KeyCode::Char('o') => {
                    self.open_link(index, links);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn open_link(&self, index: &mut usize, links: &[Link]) {
        links
            .get(*index)
            .map(|link| format!("https://{}", link.url))
            .and_then(|url| open::that(&url).err())
            .iter()
            .for_each(|e| println!("{}", e));
    }

    fn change_page(
        &self,
        index: &mut usize,
        links: &[Link],
        page: &mut Paragraph,
        scroll_offset: &mut u16,
    ) -> Result<(), MyError> {
        self.display.loading()?;
        *page = self.new_page(index, links);
        *scroll_offset = 0;
        self.draw(index, links, page, scroll_offset)?;
        Ok(())
    }

    pub fn new_page(&self, index: &mut usize, links: &[Link]) -> Paragraph<'static> {
        if let Some(link) = links.get(*index + 1) {
            preload(link);
        }
        links
            .get(*index)
            .inspect(|link|
                _ = add_history(link))
            .map(|link| get_content(link))
            .unwrap_or_else(|| Paragraph::new(Text::from(String::from("Index out of bounds"))))
    }

    fn draw(
        &self,
        index: &mut usize,
        links: &[Link],
        page: &mut Paragraph,
        scroll_offset: &mut u16,
    ) -> Result<(), MyError> {
        self.results_page(page, links.get(*index), *scroll_offset)
            .map_err(|e| MyError::DisplayError(e.to_string()))?;
        Ok(())
    }

}
