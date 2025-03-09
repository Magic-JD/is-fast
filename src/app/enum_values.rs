use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::cli::command::Cli;
use crate::search::link::Link;
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub enum App {
    Tui(TuiApp),
    Text(TextApp),
}

#[enum_dispatch(App)]
pub trait HistoryViewer {
    fn show_history(&mut self) -> Option<Link>;
}
#[enum_dispatch(App)]
pub trait PageViewer {
    fn show_page(&mut self, args: Cli);
}

#[enum_dispatch(App)]
pub trait Shutdown {
    fn shutdown(&mut self);
}

impl Shutdown for TextApp {
    fn shutdown(&mut self) {
        // There is nothing that needs to be shutdown here.
    }
}

impl Shutdown for TuiApp {
    fn shutdown(&mut self) {
        self.display.shutdown();
    }
}
