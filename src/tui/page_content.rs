use crate::config::load::Config;
use crate::search_engine::link::HtmlSource;
use crate::tui::display::Widget;
use crate::tui::display::Widget::{Block, Paragraph, Text};
use crate::tui::general_widgets::default_block;
use crate::tui::page_widgets::{draw_page_numbers, new_page};
use once_cell::sync::Lazy;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::text::Text as RText;
use ratatui::widgets::{Block as RBlock, Paragraph as RParagraph};

static PAGE_INSTRUCTIONS: &str = " Quit: q/Esc | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d/PgDn | Page Up: CTRL+u/PgUp | Next: n/→ | Back: b/← | Open in Browser: o ";
static TUI_MARGIN: Lazy<u16> = Lazy::new(Config::get_page_margin);

pub struct PageContent<'a> {
    total_area: Rect,
    widgets: (RBlock<'a>, RParagraph<'a>, RText<'a>),
    areas: (Rect, Rect, Rect),
    index: usize,
    scroll: u16,
}

impl PageContent<'_> {
    pub fn new(pages: &[HtmlSource], available_space: Rect) -> Self {
        let total_area = available_space;
        let areas = PageContent::page_area(available_space);
        let index = 0;
        let scroll = 0;
        let (title, page) = new_page(index, pages);
        let border = default_block(&title, PAGE_INSTRUCTIONS);
        let page_numbers = draw_page_numbers(index + 1, pages.len());
        let widgets = (border, page, page_numbers);
        PageContent {
            total_area,
            widgets,
            areas,
            index,
            scroll,
        }
    }

    pub fn create_widgets(
        &mut self,
        index: usize,
        scroll: u16,
        pages: &[HtmlSource],
        available_space: Rect,
    ) -> Vec<Widget<'_>> {
        if available_space != self.total_area {
            self.total_area = available_space;
            self.areas = Self::page_area(available_space);
        }
        if index != self.index {
            self.index = index;
            let (title, page) = new_page(index, pages);
            let border = default_block(&title, PAGE_INSTRUCTIONS);
            let page_numbers = draw_page_numbers(index + 1, pages.len());
            self.widgets = (border, page, page_numbers);
        }
        if scroll != self.scroll {
            self.scroll = scroll;
            self.scroll_page(scroll);
        }
        let (border, page, page_numbers) = &self.widgets;
        let (border_area, page_area, page_number_area) = &self.areas;
        vec![
            Block(border, border_area),
            Paragraph(page, page_area),
            Text(page_numbers, page_number_area),
        ]
    }

    fn scroll_page(&mut self, scroll: u16) {
        let new_page = std::mem::take(&mut self.widgets.1).scroll((scroll, 0));
        self.widgets.1 = new_page;
    }

    fn page_area(size: Rect) -> (Rect, Rect, Rect) {
        //Split vertically leaving room for the header and footer.
        let vertical_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(1),
                    Constraint::Length(size.height - 2),
                    Constraint::Length(1),
                ]
                .as_ref(),
            )
            .split(size);

        let side_margin = *TUI_MARGIN;
        let center = 100 - (side_margin * 2);

        // Split middle section horizontally to add margins to the sides.
        let horizontal_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(side_margin),
                    Constraint::Percentage(center),
                    Constraint::Percentage(side_margin),
                ]
                .as_ref(),
            )
            .split(vertical_chunks[1]);

        let page_number_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
            .split(vertical_chunks[2]);
        (size, horizontal_chunks[1], page_number_layout[1])
    }
}
