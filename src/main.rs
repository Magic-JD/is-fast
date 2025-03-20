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
use crate::cli::parser::{
    determine_cache_mode, determine_ignored, determine_nth_element, parse_pretty_print,
};
use crate::config::load::Config;
use crate::database::history_database;
use crate::search_engine::cache;
use actions::generate_config;
use clap::Parser;
use config::log::init_logger;
use crossterm::tty::IsTty;

#[derive(Clone, Debug, PartialEq)]
enum DisplayConfig {
    Margin(u16),
    Wrap,
    Title(Option<String>),
}

fn main() {
    init_logger();
    let args = Cli::parse();
    let pretty_print = parse_pretty_print(&args.output.pretty_print.join(","));
    let cache_command = determine_cache_mode(&args.cache);
    let ignored = determine_ignored(args.selection.ignore);
    let nth_element = determine_nth_element(args.selection.nth_element);
    Config::init(
        args.output.color.clone(),
        cache_command.as_ref(),
        args.history.no_history,
        pretty_print,
        args.selection.selector.clone(),
        &ignored,
        args.selection.no_block,
        nth_element,
    );
    // Generate config doesn't need a display, process and return.
    if args.task.generate_config {
        generate_config::run();
        return;
    }
    if process_clear_command(
        args.task.clear_cache,
        args.task.clear_history,
        args.task.clear_all,
    ) {
        return;
    }
    let is_piped = args.output.piped || !std::io::stdout().is_tty();
    let mut app = App::from_type(is_piped);
    app.loading();
    if args.history.history {
        if let Some(page) = app.show_history() {
            app.show_pages(&[page]);
        }
    } else {
        let page_result = prepare_pages(args.open).unwrap_or_else(|err| {
            app.shutdown_with_error(&err.to_string());
        });
        app.show_pages(&page_result);
    }
    app.shutdown();
}

fn process_clear_command(clear_cache: bool, clear_history: bool, clear_all: bool) -> bool {
    if clear_cache || clear_history || clear_all {
        if clear_cache || clear_all {
            log::debug!(
                "Clearing cache - Clear cache {}, Clear all {}",
                clear_cache,
                clear_all
            );
            cache::clear();
        }
        if clear_history || clear_all {
            log::debug!(
                "Clearing history - Clear history {}, Clear all {}",
                clear_history,
                clear_all
            );
            history_database::clear_history()
                .unwrap_or_else(|e| log::error!("Failed to clear history: {}", e));
        }
        return true;
    }
    false
}
