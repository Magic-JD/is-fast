#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![cfg_attr(not(test), deny(unused_must_use))]
#![cfg_attr(not(test), deny(clippy::todo))]

mod actions;
mod app;
mod cli;
mod config;
mod database;
mod errors;
mod pipe;
mod search;
mod transform;
mod tui;

use crate::app::enum_values::App;
use crate::app::enum_values::HistoryViewer;
use crate::app::enum_values::PageViewer;
use crate::app::enum_values::Shutdown;
use crate::app::text::TextApp;
use crate::app::tui::TuiApp;
use crate::cli::command::Cli;
use actions::generate_config;
use atty::{is, Stream};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    // Generate config doesn't need a display, process and return.
    if args.generate_config {
        generate_config::run();
        return;
    }
    let mut is_piped = args.piped;
    if !is(Stream::Stdout) {
        is_piped = true;
    }
    let mut app = if is_piped {
        App::Text(TextApp::new())
    } else {
        App::Tui(TuiApp::new())
    };
    if args.history {
        app.show_history();
    } else {
        app.show_page(args);
    }
    app.shutdown();
}
