use crate::tui::general_widgets::default_block;
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Table, TableState};
use ratatui::{Frame, Terminal};
use std::io::{stdout, Stdout};
use std::sync::{Mutex, MutexGuard};

pub struct Display {
    terminal: Mutex<Terminal<CrosstermBackend<Stdout>>>,
}

impl Display {}

const STARTUP_ERROR: &str = "Cannot properly enable TUI - shutting down.";
const SHUTDOWN_ERROR: &str =
    "Cannot properly close TUI - shutting down. Try 'reset' if there are on going terminal issues.";

impl Display {
    pub fn new() -> Self {
        // This can panic if startup not handled properly.
        enable_raw_mode().expect(STARTUP_ERROR);
        let mut out = stdout();
        execute!(out, EnterAlternateScreen).expect(STARTUP_ERROR);
        let backend = CrosstermBackend::new(out);
        let terminal = Terminal::new(backend).expect(STARTUP_ERROR);
        Display {
            terminal: Mutex::new(terminal),
        }
    }

    pub fn shutdown_with_error(&self, error: &str) -> ! {
        self.shutdown();
        eprintln!("{error}");
        std::process::exit(1);
    }

    pub fn shutdown(&self) {
        let mut terminal = self.terminal.lock().expect(SHUTDOWN_ERROR);
        execute!(terminal.backend_mut(), LeaveAlternateScreen).expect(SHUTDOWN_ERROR);
        disable_raw_mode().expect(SHUTDOWN_ERROR);
    }

    pub fn refresh(&self) {
        enable_raw_mode().expect(STARTUP_ERROR);
        let mut out = stdout();
        execute!(out, EnterAlternateScreen).expect(STARTUP_ERROR);
        self.unwrap_terminal()
            .clear()
            .unwrap_or_else(|e| self.shutdown_with_error(&e.to_string()));
    }

    pub fn height(&self) -> u16 {
        self.unwrap_terminal().get_frame().area().height
    }
    pub fn area(&self) -> Rect {
        self.unwrap_terminal().get_frame().area()
    }

    fn unwrap_terminal(&self) -> MutexGuard<Terminal<CrosstermBackend<Stdout>>> {
        let mut terminal = self
            .terminal
            .lock()
            .unwrap_or_else(|err| self.shutdown_with_error(&err.to_string()));
        terminal
            .autoresize()
            .unwrap_or_else(|err| self.shutdown_with_error(&err.to_string()));
        terminal
    }

    pub fn loading(&mut self) {
        let block = default_block("Loading...", "");
        self.unwrap_terminal()
            .draw(|frame| {
                let size = frame.area();
                frame.render_widget(block, size); // Block takes the whole area
            })
            .unwrap_or_else(|err| self.shutdown_with_error(&err.to_string()));
    }

    pub fn render(&mut self, drawables: Vec<Widget>) {
        self.unwrap_terminal()
            .draw(|frame| {
                for widget in drawables {
                    widget.render(frame);
                }
            })
            .unwrap_or_else(|err| self.shutdown_with_error(&err.to_string()));
    }
}
pub enum Widget<'a> {
    Table(&'a Table<'a>, &'a mut TableState, &'a Rect),
    Paragraph(&'a Paragraph<'a>, &'a Rect),
    Text(&'a Text<'a>, &'a Rect),
    Block(&'a Block<'a>, &'a Rect),
}

impl Widget<'_> {
    /// Renders the widget using the given frame and area.
    pub fn render(self, frame: &mut Frame) {
        match self {
            Widget::Table(table, table_state, rect) => {
                frame.render_stateful_widget(table, *rect, table_state);
            }
            Widget::Paragraph(paragraph, rect) => frame.render_widget(paragraph, *rect),
            Widget::Text(text, rect) => frame.render_widget(text, *rect),
            Widget::Block(block, rect) => frame.render_widget(block, *rect),
        }
    }
}
