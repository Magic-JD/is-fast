#![cfg_attr(not(test), deny(clippy::unwrap_used))]
#![cfg_attr(not(test), deny(clippy::panic))]
#![cfg_attr(not(test), deny(unused_must_use))]
#![cfg_attr(not(test), deny(clippy::todo))]

mod actions;
mod cli;
mod config;
mod database;
mod errors;
mod search;
mod transform;
mod tui;

use crate::actions::{direct, history};
use crate::cli::command::Cli;
use actions::generate_config;
use actions::search as search_actions;
use actions::view;
use atty::{is, Stream};
use clap::Parser;

fn main() {
    let args = Cli::parse();
    let mut is_piped = args.piped;
    if !is(Stream::Stdout) {
        is_piped = true;
    }
    if args.generate_config {
        generate_config::run();
    } else if args.history {
        history::run(is_piped);
    } else if let Some(file) = args.file {
        view::run(file, args.url, args.selector, is_piped);
    } else if let Some(url) = args.direct {
        direct::run(None, &url, args.selector, is_piped);
    } else if let Some(search_term) = args.query.map(|query| query.join(" ")) {
        if args.selector.is_some() {
            eprintln!("Selector cannot be used for a generalize search, only for a --file or --direct query");
            return;
        }
        search_actions::run(&search_term, is_piped);
    } else {
        eprintln!("No actions term provided!");
    }
}
