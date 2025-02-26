mod config;
mod error;
mod extract_formatted;
mod extract_links;
mod input;
mod link;
mod scrape;
mod syntax_highlighting;
mod ui;
mod cli;
mod run_search;

use crate::cli::Cli;
use crate::config::generate_config;
use crate::run_search::run_search;
use clap::Parser;

fn main() {
    let args = Cli::parse();
    if args.generate_config {
        generate_config();
        return;
    }
    let search_term = args.query.map(|query| query.join(" "));
    if let Some(search_term) = search_term {
        run_search(search_term);
        return;
    }
    eprintln!("No search term provided!");
}
