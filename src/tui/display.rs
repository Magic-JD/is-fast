use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use once_cell::sync::Lazy;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Span, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Table, TableState};
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::Mutex;

static TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(|| Style::default().fg(Color::Green));
static HISTORY_INSTRUCTIONS: &str =
    " Quit: Esc | Scroll Down: ↓ | Scroll Up: ↑ | Open: ↵ | Delete: Delete ";

pub struct Display {
    terminal: Mutex<Terminal<CrosstermBackend<Stdout>>>,
}

static PAGE_LAYOUT: Lazy<Layout> = Lazy::new(|| {
    Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
});

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
        eprintln!("{}", error);
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
        let paragraph = Paragraph::default();
        let paragraph = paragraph.block(default_block("Loading...", ""));
        self.draw_page(&paragraph)
    }

    pub(crate) fn draw_history(
        &self,
        table: &Table,
        row_count: u16,
        title: String,
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
            let block = default_block(&title, HISTORY_INSTRUCTIONS);
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
            frame.render_widget(block, frame.area());
            let areas = layout.split(size);
            let area = areas[1];
            frame.render_stateful_widget(table, area, state);
            frame.render_widget(
                Paragraph::new(
                    Line::from(format!(" [SEARCH] {}", user_input)).style(
                        Style::default()
                            .fg(Color::LightBlue)
                            .add_modifier(Modifier::BOLD),
                    ),
                ),
                areas[2],
            );
        })?;
        Ok(())
    }

    pub(crate) fn draw_page(&self, page: &Paragraph) -> std::io::Result<()> {
        let mut terminal = self.terminal.lock().unwrap();
        terminal.draw(|frame| {
            let size = frame.area();
            let layout = &PAGE_LAYOUT;
            let area = layout.split(size)[0];
            frame.render_widget(page, area);
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
