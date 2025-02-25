use crate::models::Link;
use once_cell::sync::Lazy;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::{HashMap, HashSet};

static IGNORED_TAGS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    [
        "script", "style", "noscript", "head", "title", "meta", "input", "button", "svg", "nav",
        "footer", "header", "aside",
    ]
    .iter()
    .cloned()
    .collect()
});

static BLOCK_ELEMENTS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    ["p", "div", "article", "section", "pre"]
        .iter()
        .cloned()
        .collect()
});

static TAG_STYLES: Lazy<HashMap<&'static str, Style>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert("h1", Style::default().add_modifier(Modifier::BOLD));
    map.insert("h2", Style::default().add_modifier(Modifier::BOLD));
    map.insert("h3", Style::default().add_modifier(Modifier::BOLD));
    map.insert("a", Style::default().fg(Color::Cyan));
    map.insert("code", Style::default().fg(Color::Red));
    map.insert("em", Style::default().add_modifier(Modifier::ITALIC));
    map.insert("i", Style::default().add_modifier(Modifier::ITALIC));
    map.insert("strong", Style::default().add_modifier(Modifier::BOLD));
    map.insert("b", Style::default().add_modifier(Modifier::BOLD));
    map.insert(
        "blockquote",
        Style::default()
            .fg(Color::Gray)
            .add_modifier(Modifier::ITALIC),
    );
    map.insert("del", Style::default().add_modifier(Modifier::CROSSED_OUT));
    map.insert("ins", Style::default().add_modifier(Modifier::UNDERLINED));
    map.insert("mark", Style::default().bg(Color::Yellow).fg(Color::Black));
    map.insert("small", Style::default().fg(Color::Gray));
    map.insert(
        "sub",
        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
    );
    map.insert(
        "sup",
        Style::default().fg(Color::Gray).add_modifier(Modifier::DIM),
    );
    map.insert("pre", Style::default().bg(Color::Black).fg(Color::White));
    map.insert("kbd", Style::default().bg(Color::DarkGray).fg(Color::White));
    map.insert("var", Style::default().fg(Color::Cyan));
    map.insert("samp", Style::default().fg(Color::Magenta));
    map.insert("u", Style::default().add_modifier(Modifier::UNDERLINED));
    map.insert("li", Style::default().add_modifier(Modifier::BOLD));
    map.insert("dt", Style::default().add_modifier(Modifier::BOLD));
    map.insert("dd", Style::default().fg(Color::Gray));
    map
});

pub fn extract_links(html: &String) -> Vec<Link> {
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
    })
    .map_err(|_| "Error: Could not parse selector")?;
    Ok(Paragraph::new(Text::from(
        Html::parse_document(&res)
            .select(&selector)
            .flat_map(|e| convert_to_text(e))
            .collect::<Vec<Line>>(),
    )))
}

fn convert_to_text(element: ElementRef) -> Vec<Line<'static>> {
    let tag_name = element.value().name();

    if IGNORED_TAGS.contains(tag_name) {
        return Vec::new();
    }

    let style = TAG_STYLES
        .get(tag_name)
        .unwrap_or(&Style::default())
        .clone();

    let mut lines = Vec::new();

    element.children().for_each(|node| match node.value() {
        Node::Text(text) => {
            if !text.trim().is_empty() {
                let mut current_lines = text
                    .lines()
                    .map(|line| Line::from(Span::styled(line.to_string(), style)))
                    .collect::<Vec<Line>>();

                let last_line_o = lines.pop();
                // If there is no previous line, then we don't need to do anything
                if last_line_o.is_none() {
                    lines.extend(current_lines);
                    return;
                }
                let mut spans = Vec::new();
                let last_line = last_line_o.unwrap();
                spans.extend(last_line.spans.clone());
                spans.extend(current_lines.first().unwrap().spans.clone());
                lines.push(Line::from(spans));
                lines.extend(current_lines.drain(1..));
            }
        }
        Node::Element(_) => ElementRef::wrap(node).iter().for_each(|element| {
            let mut element_lines = convert_to_text(*element);
            if element_lines.is_empty() {
                return;
            }
            if BLOCK_ELEMENTS.contains(element.value().name()) {
                lines.extend(element_lines);
                lines.push(Line::from(Span::styled("", style)));
                return;
            }
            // If it is not the case that the element should be a block, we should join it to the
            // previous line.
            let last_line_o = lines.pop();
            // If there is no previous line, then we don't need to do anything
            if last_line_o.is_none() {
                lines.extend(element_lines);
                return;
            }
            let mut spans = Vec::new();
            let last_line = last_line_o.unwrap();
            spans.extend(last_line.spans.clone());
            spans.extend(element_lines.first().unwrap().spans.clone());
            lines.push(Line::from(spans));
            lines.extend(element_lines.drain(1..));
        }),
        _ => {}
    });

    lines
}
