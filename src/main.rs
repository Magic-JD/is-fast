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
mod search_engine;
mod transform;
mod tui;

use crate::actions::prepare_pages::prepare_pages;
use crate::app::enum_values::App;
use crate::app::enum_values::AppFunctions;
use crate::app::enum_values::HistoryViewer;
use crate::app::enum_values::PageViewer;
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
    let is_piped = args.piped || !is(Stream::Stdout);
    let mut app = App::from_type(is_piped);
    app.loading();
    if args.history && !args.last {
        if let Some(page) = app.show_history() {
            app.show_pages(&[page]);
        }
    } else {
        let page_result = prepare_pages(args).unwrap_or_else(|err| {
            app.shutdown_with_error(&err.to_string());
        });
        app.show_pages(&page_result);
    }
    app.shutdown();
}
