use crate::models::Link;
use ratatui::text::{Line, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Selector};

pub fn extract_links(html: &String) -> Vec<Link> {
    let document = Html::parse_document(&html);
    let selector_title = Selector::parse("a.result__a").unwrap();
    let selector_url = Selector::parse("a.result__url").unwrap();

    document
        .select(&selector_title)
        .zip(document.select(&selector_url))
        .take(5)
        .map(|(title, url)| Link::new(
            title.text().collect::<Vec<_>>().join(" ").trim().to_owned(),
            url.text().collect::<Vec<_>>().join(" ").trim().to_owned()
        ))
        .collect()
}

pub fn extract_page_content(url: &String, res: &String) -> Result<Paragraph<'static>, String> {
    let selector = Selector::parse(match url {
        u if u.contains("en.wikipedia.org") => "p",
        u if u.contains("www.baeldung.com") => ".post-content",
        u if u.contains("www.w3schools.com") => "#main",
        u if u.contains("linuxhandbook.com") => "article",
        u if u.contains("docs.spring.io") => "article",
        u if u.contains("stackoverflow.com") => ".js-post-body, .user-details, .comment-body",
        u if u.contains("github.com") => ".markdown-body",
        _ => "body",
    }).map_err(|_| "Error: Could not parse selector")?;
    Ok(Paragraph::new(Text::from(Html::parse_document(&res)
        .select(&selector)
        .map(|e| convert_to_text(e))
        .map(|text| text.clone())
        .flat_map(|text| text.lines)
        .collect::<Vec<Line>>())
    ))
}

fn convert_to_text(input: ElementRef) -> Text<'static> {
    Text::from("TEST")
}
