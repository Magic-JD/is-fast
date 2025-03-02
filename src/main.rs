mod actions;
mod cli;
mod config;
mod database;
mod errors;
mod formatting;
mod links;
mod scrapers;
mod stout;
mod tui;

use crate::actions::{direct, history};
use crate::cli::command::Cli;
use actions::generate_config;
use actions::search;
use actions::view;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    if args.generate_config {
        generate_config::run();
        return;
    }
    if args.history {
        history::run();
        return;
    }
    if let Some(file) = args.file {
        view::run(file, args.url, args.piped);
        return;
    }
    if let Some(url) = args.direct {
        direct::run(None, url, args.piped);
        return;
    }
    let search_term = args.query.map(|query| query.join(" "));
    if let Some(search_term) = search_term {
        search::run(search_term);
        return;
    }
    eprintln!("No actions term provided!");
}
