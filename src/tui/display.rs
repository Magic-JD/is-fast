use crate::config::load::Config;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use once_cell::sync::Lazy;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout};
use ratatui::prelude::{Modifier, Span, Style};
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Borders, Paragraph, Table, TableState};
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::Mutex;

static TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(Config::get_border_color);
static TUI_MARGIN: Lazy<u16> = Lazy::new(Config::get_page_margin);
static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete ";

static PAGE_INSTRUCTIONS: &str = " Quit: q/Esc | Scroll Down: j/↓ | Scroll Up: k/↑ | Page Down: CTRL+d | Page Up: CTRL+u | Next: n/→ | Back: b/← | Open in Browser: o ";
static TEXT_COLOR: Lazy<Style> = Lazy::new(Config::get_text_color);

pub struct Display {
    terminal: Mutex<Terminal<CrosstermBackend<Stdout>>>,
}

impl Display {
    pub(crate) fn new() -> Self {
        // This can panic if startup not handled properly.
        enable_raw_mode().unwrap();
        let mut out = stdout();
        execute!(out, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend).unwrap();
        Display {
            terminal: Mutex::new(terminal),
        }
    }

    pub fn shutdown_with_error(&mut self, error: &str) -> ! {
        self.shutdown();
        eprintln!("{error}");
        std::process::exit(1);
    }

    pub(crate) fn shutdown(&mut self) {
        let mut terminal = self.terminal.lock().unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        disable_raw_mode().unwrap();
    }

    pub fn height(&self) -> u16 {
        self.terminal.lock().unwrap().get_frame().area().height
    }

    pub fn loading(&self) -> std::io::Result<()> {
        let block = default_block("Loading...", "");
        let mut terminal = self.terminal.lock().unwrap();
        terminal.draw(|frame| {
            let size = frame.area();
            frame.render_widget(block, size); // Block takes the whole area
        })?;
        Ok(())
    }

    pub(crate) fn draw_history(
        &self,
        table: &Table,
        row_count: u16,
        title: &str,
        state: &mut TableState,
        user_input: &mut String,
        should_reset_position: bool,
    ) -> std::io::Result<()> {
        let mut terminal = self.terminal.lock().unwrap();
        terminal.draw(|frame| {
            let size = frame.area();
            let available_height = size.height;
            let table_height = row_count.min(available_height);
            if should_reset_position {
                //Set the offset to 0, then select the last item to correctly scroll the page.
                *state.offset_mut() = 0;
                state.select_last();
            }
            let block = default_block(title, HISTORY_INSTRUCTIONS);
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Length(table_height),
                        Constraint::Length(2),
                    ]
                    .as_ref(),
                );
            let areas = layout.split(size);
            let area = areas[1];
            let search_bar_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(areas[2]);
            frame.render_widget(block, frame.area());
            frame.render_stateful_widget(table, area, state);
            frame.render_widget(
                Paragraph::new(
                    Line::from(format!(" [SEARCH] {user_input}"))
                        .style(TEXT_COLOR.add_modifier(Modifier::BOLD)),
                ),
                search_bar_layout[0],
            );
            frame.render_widget(
                Text::from(
                    Line::from(count_result_text(row_count))
                        .style(TEXT_COLOR.add_modifier(Modifier::BOLD)),
                ),
                search_bar_layout[1],
            );
        })?;
        Ok(())
    }

    pub(crate) fn draw_page(
        &self,
        page: &Paragraph,
        title: &str,
        index: usize,
        pages: usize,
    ) -> std::io::Result<()> {
        let mut terminal = self.terminal.lock().unwrap();
        terminal.draw(|frame| {
            let size = frame.area();
            let block = default_block(title, PAGE_INSTRUCTIONS);
            frame.render_widget(block, size); // Block takes the whole area

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
            frame.render_widget(page, horizontal_chunks[1]);
            frame.render_widget(
                Text::from(Line::styled(
                    format!(" [{index}/{pages}] "),
                    *TUI_BORDER_COLOR,
                ))
                .alignment(Alignment::Right),
                page_number_layout[1],
            );
        })?;
        Ok(())
    }
}
pub fn default_block(title: &str, instructions: &str) -> Block<'static> {
    Block::default()
        .title(tui_border_span(title))
        .title_bottom(tui_border_span(instructions))
        .borders(Borders::TOP)
        .style(*TUI_BORDER_COLOR)
}

fn tui_border_span(text: &str) -> Span<'static> {
    Span::styled(
        text.to_string(),
        (*TUI_BORDER_COLOR).add_modifier(Modifier::BOLD),
    )
}

fn count_result_text(row_count: u16) -> String {
    if row_count == 1 {
        format!("{row_count} result")
    } else {
        format!("{row_count} results")
    }
}
