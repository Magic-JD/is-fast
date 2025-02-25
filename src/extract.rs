use std::collections::HashSet;
use ratatui::style::{Color, Style};
use crate::models::Link;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Node, Selector};
use once_cell::sync::Lazy;

static IGNORED_TAGS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "script", "style", "noscript", "head", "title", "meta", "input",
        "button", "svg", "nav", "footer", "header", "aside"
    ].iter().cloned().collect()
});

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
        .filter(|e| e.is_some())
        .map(|e| e.unwrap())
        .map(|text| text.clone())
        .flat_map(|text| text.lines)
        .collect::<Vec<Line>>())
    ))
}

fn convert_to_text(element: ElementRef) -> Option<Text<'static>> {
    let tag_name = element.value().name();

    if IGNORED_TAGS.contains(tag_name) {
        return None;
    }

    let mut style = Style::default();

    match tag_name {
        "h1" | "h2" | "h3" => {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        "a" => {
            style = style.fg(Color::Cyan);
        }
        "code" => {
            style = style.fg(Color::Red);
        }
        "em" | "i" => {
            style = style.add_modifier(ratatui::style::Modifier::ITALIC);
        }
        "strong" | "b" => {
            style = style.add_modifier(ratatui::style::Modifier::BOLD);
        }
        _ => {}
    }

    let children_lines: Vec<Line> = element.children().flat_map(|node| {
        match node.value() {
            Node::Text(text) => {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    vec![Line::from(Span::styled(trimmed.to_string(), style))]
                } else {
                    vec![]
                }
            }
            Node::Element(_) => {
                ElementRef::wrap(node)
                    .map(|e| convert_to_text(e))
                    .filter(|e| e.is_some())
                    .map(|child| child.unwrap().lines.into_iter().collect::<Vec<Line>>())
                    .unwrap_or_else(Vec::new)
            }
            _ => vec![],
        }
    }).collect();

    Some(Text::from(children_lines))
}
