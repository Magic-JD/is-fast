use crate::config::load::Config;
use crate::transform::scrape::scrape;
use scraper::{Html, Selector};

#[derive(Clone)]
pub struct Link {
    pub url: String,
    pub title: String,
    pub selector: String,
}
impl Link {
    pub fn new(title: String, url: String, selector: String) -> Self {
        Self {
            url,
            title,
            selector,
        }
    }
}
pub fn get_links(search_term: &String) -> Vec<Link> {
    scrape(&format!(
        "https://html.duckduckgo.com/html/?q={}",
        &search_term
    ))
    .map(|html| from_html(&html))
    .unwrap_or_else(|_| vec![])
}

fn from_html(html: &str) -> Vec<Link> {
    let document = Html::parse_document(html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .map(|(title, url)| {
            let url = url.text().collect::<Vec<_>>().join(" ").trim().to_owned();
            Link::new(
                title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
                url.clone(),
                Config::get_selectors(&url),
            )
        })
        .collect()
}
