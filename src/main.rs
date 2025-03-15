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
use crate::config::load::Config;
use crate::database::connect::clear_history;
use crate::search_engine::cache;
use actions::generate_config;
use atty::{is, Stream};
use clap::Parser;

#[derive(Clone, Debug)]
enum DisplayConfig {
    Margin(u16),
    Wrap,
    Title(String),
}

#[derive(Clone, Debug)]
enum CacheCommand {
    Cache,
    Disable,
    Flash,
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    let pretty_print = parse_pretty_print(&args.pretty_print.join(","));
    let cache_command = match (args.cache, args.no_cache, args.flash_cache) {
        (true, false, false) => Some(CacheCommand::Cache),
        (false, true, false) => Some(CacheCommand::Disable),
        (false, false, true) => Some(CacheCommand::Flash),
        (_, _, _) => None,
    };
    Config::init(
        args.color.clone(),
        cache_command,
        args.no_history,
        pretty_print,
    );
    // Generate config doesn't need a display, process and return.
    if args.generate_config {
        generate_config::run();
        return;
    }
    if process_clear_command(&args) {
        return;
    }
    let is_piped = args.piped || !is(Stream::Stdout);
    let mut app = App::from_type(is_piped);
    app.loading();
    if args.history {
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

fn parse_pretty_print(line: &str) -> Vec<DisplayConfig> {
    if line.is_empty() {
        return vec![];
    }
    line.split(',')
        .filter_map(|s| {
            let mut parts = s.split(':');
            match (parts.next(), parts.next()) {
                (Some("wrap"), None) => Some(DisplayConfig::Wrap),
                (Some("margin"), Some(value)) => {
                    value.parse::<u16>().ok().map(DisplayConfig::Margin)
                }
                (Some("title"), Some(value)) => Some(DisplayConfig::Title(value.to_string())),
                _ => {
                    log::error!("Invalid display configuration: {}", s);
                    None
                }
            }
        })
        .collect()
}

fn process_clear_command(args: &Cli) -> bool {
    if args.clear_cache || args.clear_history || args.clear_all {
        if args.clear_cache || args.clear_all {
            log::debug!(
                "Clearing cache - Clear cache {}, Clear all {}",
                args.clear_cache,
                args.clear_all
            );
            cache::clear();
        }
        if args.clear_history || args.clear_all {
            log::debug!(
                "Clearing history - Clear history {}, Clear all {}",
                args.clear_history,
                args.clear_all
            );
            clear_history().unwrap_or_else(|e| log::error!("Failed to clear history: {}", e));
        }
        return true;
    }
    false
}
