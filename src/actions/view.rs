use crate::config::load::Config;
use crate::search::link::Link;
use crate::transform::page::PageExtractor;
use crate::tui::browser::Browser;

pub fn run(file: String, url: Option<String>, selection: Option<String>, piped: bool) {
    let url = url.unwrap_or_else(|| file.clone());
    let selection_tag = selection.unwrap_or_else(|| Config::get_selectors(&url));
    let link = Link::new(file, url, selection_tag);
    if piped {
        let text = PageExtractor::from_file().get_plain_text(&link);
        println!("{}", text);
        return;
    }
    Browser::new().browse(vec![link], PageExtractor::from_file(), false);
}
