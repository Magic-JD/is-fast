use scraper::{Html, Selector};
use crate::links::link::Link;

pub fn from_html(html: &String) -> Vec<Link> {
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(title, url)| {
            Link::new(
                title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
                url.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
            )
        })
        .collect()
}
