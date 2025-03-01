use crate::links::link::Link;
use crate::scrapers::scrape::scrape;
use scraper::{Html, Selector};

pub fn from_html(html: &String) -> Vec<Link> {
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(title, url)| {
            let url = url.text().collect::<Vec<_>>().join(" ").trim().to_owned();
            Link::new(
                title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
                url.clone(),
                move || scrape(&format!("https://{}", url).to_string())
            )
        })
        .collect()
}
