use crate::links::link::Link;
use crate::tui::render;
use std::fs;

pub fn run(file: String, url: Option<String>) {
    let url = url.unwrap_or_else(|| file.clone());
    let html = fs::read_to_string(&file).unwrap();
    render::show(&vec![Link::new(file, url, move || Ok(html.clone()))]);
}
