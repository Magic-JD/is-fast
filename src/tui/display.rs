use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use once_cell::sync::Lazy;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::{Color, Modifier, Span, Style};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph, Table, TableState, Wrap};
use ratatui::Terminal;
use std::io::{stdout, Stdout};
use std::sync::Mutex;

const TUI_BORDER_COLOR: Lazy<Style> = Lazy::new(|| Style::default().fg(Color::Green));

pub struct Display {
    terminal: Mutex<Terminal<CrosstermBackend<Stdout>>>,
    instructions: String,
}

impl Display {
    pub(crate) fn new(instructions: String) -> Self {
        // This can panic if startup not handled properly.
        enable_raw_mode().unwrap();
        let mut out = stdout();
        execute!(out, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend).unwrap();
        Display {
            terminal: Mutex::new(terminal),
            instructions,
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
        self.draw(&Paragraph::default(), " Loading...".to_string(), 0)
    }

    pub(crate) fn draw_table(
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
            let block = self.default_block(&title);
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
            let new_table = table.clone();
            let areas = layout.split(size);
            let area = areas[1];
            frame.render_stateful_widget(new_table, area, state);
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

    fn default_block<'a>(&'a self, title: &'a str) -> Block<'a> {
        Block::default()
            .title(self.tui_border_span(title))
            .title_bottom(self.tui_border_span(&self.instructions))
            .borders(Borders::TOP)
            .style(TUI_BORDER_COLOR.clone())
    }

    pub(crate) fn draw(
        &self,
        page: &Paragraph,
        title: String,
        scroll_offset: u16,
    ) -> std::io::Result<()> {
        let mut terminal = self.terminal.lock().unwrap();
        terminal.clear()?;
        terminal.draw(|frame| {
            let size = frame.area();
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(100)].as_ref());
            let area = layout.split(size)[0];
            let block = self.default_block(&title);
            let paragraph = Paragraph::from(page.clone())
                .block(block)
                .style(Style::default().fg(Color::White))
                .wrap(Wrap { trim: false })
                .scroll((scroll_offset, 0));

            frame.render_widget(paragraph, area);
        })?;
        Ok(())
    }

    fn tui_border_span<'a>(&self, text: &'a str) -> Span<'a> {
        Span::styled(text, TUI_BORDER_COLOR.clone().add_modifier(Modifier::BOLD))
    }
}
