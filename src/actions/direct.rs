use crate::config::load::Config;
use crate::search::link::Link;
use crate::transform::page::PageExtractor;
use crate::tui::browser::Browser;

pub fn run(title: Option<String>, url: String, selector: Option<String>, piped: bool) {
    let selection_tag = selector.unwrap_or_else(|| Config::get_selectors(&url));
    let link = Link::new(
        title.unwrap_or_default(),
        url.to_string(),
        selection_tag.clone(),
    );
    if piped {
        let text_only = PageExtractor::from_url().get_plain_text(&link);
        println!("{}", text_only);
        return;
    }
    Browser::new().browse(vec![link], PageExtractor::from_url(), false);
}
