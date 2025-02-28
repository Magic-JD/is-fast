mod scrapers;
mod formatting;
mod links;
mod tui;
mod config;
mod errors;
mod actions;
mod cli;

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
    if let Some(file) = args.file {
        view::run(file, args.url);
        return;

    }
    let search_term = args.query.map(|query| query.join(" "));
    if let Some(search_term) = search_term {
        search::run(search_term);
        return;
    }
    eprintln!("No actions term provided!");
}
