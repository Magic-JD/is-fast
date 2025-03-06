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
use clap::Parser;

fn main() {
    let args = Cli::parse();
    if args.generate_config {
        generate_config::run();
    } else if args.history {
        history::run();
    } else if let Some(file) = args.file {
        view::run(file, args.url, args.selector, args.piped);
    } else if let Some(url) = args.direct {
        direct::run(None, url, args.selector, args.piped);
    } else if let Some(search_term) = args.query.map(|query| query.join(" ")) {
        search_actions::run(search_term);
    } else {
        eprintln!("No actions term provided!");
    }
}
