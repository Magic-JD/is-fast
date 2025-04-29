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
mod page;
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
use crate::config::color_conversion::Style;
use crate::config::load::Config;
use crate::database::history_database;
use crate::errors::error::IsError;
use crate::search_engine::cache;
use actions::generate_config;
use clap::Parser;
use config::log::init_logger;
use crossterm::tty::IsTty;
use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
enum DisplayConfig {
    Margin(u16),
    Wrap,
    Title(Option<String>),
}

fn main() {
    let args = Cli::parse();
    init_logger(args.log);
    let pretty_print = parse_pretty_print(&args.output.pretty_print.join(","));
    let cache_command = determine_cache_mode(&args.cache);
    let ignored = determine_ignored(args.selection.ignore);
    let nth_element = determine_nth_element(args.selection.nth_element);
    let styles = determine_styles(args.output.style_element);
    Config::init(
        args.output.color.clone(),
        cache_command.as_ref(),
        args.history.no_history,
        pretty_print,
        args.selection.selector.clone(),
        &ignored,
        args.selection.no_block,
        nth_element,
        &styles,
        args.open.site.clone(),
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

fn determine_styles(styles: Vec<String>) -> Vec<(String, Style)> {
    styles
        .into_iter()
        .filter_map(|style| {
            let mut split = style.split(':');
            if let Some(key) = split.next() {
                if let Ok(tag_content) = split
                    .next()
                    .ok_or(IsError::TagStyleError(format!(
                        "No configuration for {key}",
                    )))
                    .and_then(Style::from_str)
                {
                    return Some((key.to_string(), tag_content));
                }
            }
            None
        })
        .collect()
}

fn process_clear_command(clear_cache: bool, clear_history: bool, clear_all: bool) -> bool {
    if clear_cache || clear_history || clear_all {
        if clear_cache || clear_all {
            log::debug!("Clearing cache - Clear cache {clear_cache}, Clear all {clear_all}");
            cache::clear();
        }
        if clear_history || clear_all {
            log::debug!("Clearing history - Clear history {clear_history}, Clear all {clear_all}");
            history_database::clear_history()
                .unwrap_or_else(|e| log::error!("Failed to clear history: {e}"));
        }
        return true;
    }
    false
}
