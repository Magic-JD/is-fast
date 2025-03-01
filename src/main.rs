mod scrapers;
mod formatting;
mod links;
mod tui;
mod config;
mod errors;
mod actions;
mod cli;
mod database;

use crate::cli::command::Cli;
use actions::generate_config;
use actions::search;
use actions::view;
use clap::Parser;
use crate::actions::{direct, history};
use crate::actions::history::run_open;

fn main() {
    let args = Cli::parse();
    if args.generate_config {
        generate_config::run();
        return;
    }
    if args.history {
        if let Some(index) = args.history_select {
            run_open(index);
            return;
        }
        history::run();
        return;
    }
    if let Some(file) = args.file {
        view::run(file, args.url);
        return;

    }
    if let Some(url) = args.direct {
        direct::run(None, url);
        return;
    }
    let search_term = args.query.map(|query| query.join(" "));
    if let Some(search_term) = search_term {
        search::run(search_term);
        return;
    }
    eprintln!("No actions term provided!");
}
