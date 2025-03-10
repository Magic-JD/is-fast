use crate::app::enum_values::App::{Text, Tui};
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::search_engine::link::PageSource;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub enum App {
    Tui(TuiApp),
    Text(TextApp),
}

impl App {
    pub fn from_type(piped: bool) -> Self {
        if piped {
            Text(TextApp::new())
        } else {
            Tui(TuiApp::new())
        }
    }
}

#[enum_dispatch(App)]
pub trait HistoryViewer {
    fn show_history(&mut self) -> Option<PageSource>;
}
#[enum_dispatch(App)]
pub trait PageViewer {
    fn show_pages(&mut self, pages: &[PageSource]);
}

#[enum_dispatch(App)]
pub trait Shutdown {
    fn shutdown(&mut self);
    fn shutdown_with_error(&mut self, error: &str) -> !;
}

impl Shutdown for TextApp {
    fn shutdown(&mut self) {
        // There is nothing that needs to be shutdown here.
    }

    fn shutdown_with_error(&mut self, error: &str) -> ! {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

impl Shutdown for TuiApp {
    fn shutdown(&mut self) {
        self.display.shutdown();
    }

    fn shutdown_with_error(&mut self, error: &str) -> ! {
        self.display.shutdown_with_error(error);
    }
}
