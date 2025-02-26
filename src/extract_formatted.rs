use crate::config::Config;
use crate::syntax_highlighting::highlight_code;
use once_cell::sync::Lazy;
use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Node, Selector};
use std::collections::{HashMap, HashSet};

static IGNORED_TAGS: Lazy<&HashSet<String>> = Lazy::new(|| Config::get_ignored_tags());
static BLOCK_ELEMENTS: Lazy<&HashSet<String>> = Lazy::new(|| Config::get_block_elements());
static TAG_STYLES: Lazy<&HashMap<String, Style>> = Lazy::new(Config::get_styles);



pub fn extract_page_content(url: &String, res: &String) -> Result<Paragraph<'static>, String> {
    let selection_tag = Config::get_selectors()
        .iter()
        .find(|(k, _)| url.contains(*k))
        .map(|(_, v)| v.clone())
        .unwrap_or("body".to_string());
    let selector =
        Selector::parse(&selection_tag).map_err(|_| "Error: Could not parse selector")?;
    let mut lines = Html::parse_document(&res)
        .select(&selector)
        .flat_map(|e| convert_to_text(e))
        .collect::<Vec<Line>>();
    lines.dedup();
    while let Some(first) = lines.first() {
        if first
            .spans
            .iter()
            .all(|span| span.content.trim().is_empty())
        {
            lines.remove(0);
        } else {
            break;
        }
    }
    if lines.is_empty() {
        return Err("No content found".to_string());
    }
    Ok(Paragraph::new(Text::from(lines)))
}

fn convert_to_text(element: ElementRef) -> Vec<Line<'static>> {
    let tag_name = element.value().name();

    if tag_name == "br" {
        return vec![
            Line::default(),
            Line::from(Span::styled("", Style::default())),
        ];
    }

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
                    .split_inclusive('\n')
                    .map(|line| Line::from(Span::styled(line.to_string(), style)))
                    .collect::<Vec<Line>>();
                merge_with_previous_line(&mut lines, &mut current_lines);
            }
        }
        Node::Element(_) => ElementRef::wrap(node).iter().for_each(|element| {
            let mut element_lines = convert_to_text(*element);
            if element_lines.is_empty() {
                return;
            }
            if BLOCK_ELEMENTS.contains(element.value().name()) {
                lines.extend(element_lines);
                return;
            }
            merge_with_previous_line(&mut lines, &mut element_lines);
        }),
        _ => {}
    });

    if BLOCK_ELEMENTS.contains(tag_name) && !lines.is_empty() {
        lines.insert(0, Line::default());
        lines.push(Line::default());
    }
    if tag_name == "code" {
        let option = element.value().attr("class");
        let language_type = option
            .map(|class_attr| {
                class_attr
                    .split_whitespace()
                    .filter(|class_name| {
                        class_name.starts_with("language-") || class_name.starts_with("lang-")
                    })
                    .map(|class_name| class_name.replace("language-", "").replace("lang-", ""))
                    .last()
                    .unwrap_or_else(|| "not-found".to_string())
            })
            .unwrap_or_else(|| "not-found".to_string());
        let code_text = lines
            .iter()
            .map(|line| line.spans.iter().map(|span| span.content.clone()).collect())
            .collect::<Vec<String>>()
            .join("");
        return highlight_code(&code_text, &language_type); // Work out how to determine the language
    }
    lines
}

fn merge_with_previous_line(lines: &mut Vec<Line<'static>>, new_lines: &mut Vec<Line<'static>>) {
    if new_lines.is_empty() {
        return;
    }
    if let Some(last_line) = lines.pop() {
        let mut spans = last_line.spans.clone();
        spans.extend(new_lines.first().unwrap().spans.clone());
        lines.push(Line::from(spans));
        lines.extend(new_lines.drain(1..));
    } else {
        lines.extend(new_lines.drain(..));
    }
}
