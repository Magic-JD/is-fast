use crate::config::load::{Config, FormatConfig};
use crate::transform::syntax_highlight::highlight_code;
use ratatui::style::{Style, Styled};
use ratatui::text::{Line, Span};
use scraper::{ElementRef, Node};

pub struct Formatter {
    config: FormatConfig,
}

impl Formatter {
    pub fn new() -> Formatter {
        Formatter {
            config: Config::get_format_config(),
        }
    }

    pub fn to_display(&self, element: ElementRef) -> Vec<Line<'static>> {
        log::trace!("Converting element to display lines: {:?}", element);
        let mut lines = self
            .to_lines(element, element.value().name() == "pre")
            .into_iter()
            .map(standardize_empty)
            .collect::<Vec<Line>>();
        lines.dedup();
        lines
    }

    fn to_lines(&self, element: ElementRef, pre_formatted: bool) -> Vec<Line<'static>> {
        if is_hidden(&element) {
            return vec![];
        }

        let tag_name = element.value().name();

        if self.config.is_element_ignored(&element) {
            return vec![];
        }

        if tag_name == "br" {
            return vec![
                // Must return 2 lines - the first will be merged back into the previous line,
                // and the second will be the start of the next line.
                // Must be treated differently to block elements as it requires no empty line.
                Line::default(),
                Line::from(Span::from("")),
            ];
        }

        if tag_name == "code" {
            // Handle code differently due to performance issues.
            let language_type = extract_language_type(element);
            let code_text = extract_code(element);
            return highlight_code(&code_text, &language_type);
        }

        let style = self.config.style_for_tag(tag_name);

        let mut lines = Vec::new();

        if tag_name == "img" {
            // Show there is an image without rendering the image.
            lines.push(create_optionally_styled_line("IMAGE", style));
        } else {
            lines = self.extract_lines(element, pre_formatted || tag_name == "pre", style);
        }

        if tag_name == "li" {
            if let Some(line) = lines.first_mut() {
                line.spans.insert(
                    0,
                    Span::styled("• ", style.copied().unwrap_or_else(Style::default)),
                );
            }
        }

        if lines.is_empty() {
            return vec![];
        }

        if self.config.is_block_element(&element) {
            // Relies on the above line to verify lines isn't empty
            if let Some(styled) = style {
                lines = lines
                    .into_iter()
                    .map(|line| line.set_style(*styled))
                    .collect();
            }
            lines.insert(0, Line::default());
            lines.push(Line::default());
        }

        lines
    }

    fn extract_lines(
        &self,
        element: ElementRef,
        pre_formatted: bool,
        style: Option<&Style>,
    ) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        element.children().for_each(|node| match node.value() {
            Node::Text(text) => {
                if pre_formatted {
                    let current_lines = text
                        .split('\n')
                        .map(|line| create_optionally_styled_line(line, style))
                        .collect::<Vec<Line>>();
                    merge_with_previous_line(&mut lines, current_lines);
                } else if !text.trim().is_empty() {
                    let current_lines = vec![create_optionally_styled_line(
                        &text.replace('\n', " "),
                        style,
                    )];
                    merge_with_previous_line(&mut lines, current_lines);
                }
            }
            Node::Element(_) => ElementRef::wrap(node).iter().for_each(|element| {
                let element_lines = self.to_lines(*element, pre_formatted);
                if element_lines.is_empty() {
                    return;
                }
                if self.config.is_block_element(element) {
                    lines.extend(element_lines);
                    return;
                }
                merge_with_previous_line(&mut lines, element_lines);
            }),
            _ => {}
        });
        lines
    }
}

fn extract_code(element: ElementRef) -> String {
    let mut fragments = Vec::new();
    for node in element.children() {
        match node.value() {
            Node::Text(text) => {
                fragments.push(text.to_string());
            }
            Node::Element(_) => {
                if let Some(child) = ElementRef::wrap(node) {
                    fragments.push(extract_code(child));
                }
            }
            _ => {}
        }
    }
    fragments.join("")
}

fn extract_language_type(element: ElementRef) -> String {
    element
        .value()
        .attr("class")
        .into_iter()
        .flat_map(|class_attr| class_attr.split_whitespace())
        .filter(|class_name| class_name.starts_with("language-") || class_name.starts_with("lang-"))
        .map(|class_name| class_name.replace("language-", "").replace("lang-", ""))
        .next()
        .unwrap_or_else(|| "not-found".to_string())
}

fn is_hidden(element: &ElementRef) -> bool {
    if element.value().attr("hidden") == Some("true") {
        return true;
    }
    if let Some(style) = element.value().attr("style") {
        if style.contains("display: none") || style.contains("visibility: hidden") {
            return true;
        }
    }
    if element.value().attr("aria-hidden") == Some("true") {
        return true;
    }
    false
}

fn standardize_empty(line: Line) -> Line {
    if line.spans.is_empty() || line.spans.iter().all(|span| span.content.trim().is_empty()) {
        Line::default()
    } else {
        line
    }
}

fn create_optionally_styled_line(content: &str, style: Option<&Style>) -> Line<'static> {
    if let Some(style) = style {
        Line::from(Span::styled(content.to_string(), *style))
    } else {
        Line::from(Span::from(content.to_string()))
    }
}

fn merge_with_previous_line(lines: &mut Vec<Line<'static>>, mut new_lines: Vec<Line<'static>>) {
    if let (Some(prev_end), Some(new_start)) = (lines.last_mut(), new_lines.first_mut()) {
        if let (Some(end), Some(start)) = (prev_end.spans.last(), new_start.spans.first()) {
            // Text from two different elements should almost always have a space - only exception for punctuation.
            // However often that space is achieved through css rather than the text.
            // Therefore, we check this manually.
            if !(start.content.is_empty()
                || end.content.is_empty()
                || end.content.ends_with(' ')
                || start.content.starts_with(' ')
                || matches!(start.content.chars().next(), Some(c) if ".:;,)}]>\"\\`'/".contains(c))
                || matches!(end.content.chars().last(), Some(c) if "<[{(`'\"\\/".contains(c)))
            {
                prev_end.spans.push(Span::from(" "));
            }
            prev_end.spans.append(&mut new_start.spans);
            new_lines.drain(..1);
        }
    }

    lines.append(&mut new_lines);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::style::{Color, Modifier};
    use ratatui::text::Text;
    use scraper::{Html, Selector};
    use std::collections::{HashMap, HashSet};

    impl Formatter {
        fn test(config: FormatConfig) -> Self {
            Self { config }
        }
    }

    #[test]
    fn test_to_display_simple_html() {
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <h1>Hello, World!</h1>
                    <p>This is a <strong>test</strong>.</p>
                </body>
            </html>
        "#;

        let formatter = Formatter::new();
        let binding = Html::parse_document(html);
        let result = Text::from(
            binding
                .select(&Selector::parse("body").unwrap())
                .flat_map(|element| formatter.to_display(element))
                .collect::<Vec<Line>>(),
        );

        let expected = Text::from(vec![
            Line::default(),
            Line::from(Span::styled(
                "Hello, World!",
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .set_style(Style::default().add_modifier(Modifier::BOLD)),
            Line::default(),
            Line::from_iter([
                Span::from("This is a "),
                Span::styled("test", Style::default().add_modifier(Modifier::BOLD)),
                Span::from("."),
            ]),
            Line::default(),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_with_preformatted_code() {
        let html = r#"
            <html>
                <body>
                    <pre><code class="language-rust">fn main() { println!("Hello, Rust!"); }</code></pre>
                </body>
            </html>
        "#;

        let formatter = Formatter::new();
        let binding = Html::parse_document(html);
        let result = Text::from(
            binding
                .select(&Selector::parse("body").unwrap())
                .flat_map(|element| formatter.to_display(element))
                .collect::<Vec<Line>>(),
        );

        // Since `highlight_code` transforms the code, we check if the output contains expected text
        assert!(result
            .to_string()
            .contains("fn main() { println!(\"Hello, Rust!\"); }"));

        let expected = Text::from(vec![
            Line::default(),
            Line::from_iter([
                Span::styled("fn", Style::default().fg(Color::Rgb(180, 142, 173))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("main", Style::default().fg(Color::Rgb(143, 161, 179))),
                Span::styled("(", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(")", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("{", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("println!", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("(", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("\"", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(
                    "Hello, Rust!",
                    Style::default().fg(Color::Rgb(163, 190, 140)),
                ),
                Span::styled("\"", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(")", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(";", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("}", Style::default().fg(Color::Rgb(192, 197, 206))),
            ]),
            Line::default(),
        ]);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_no_content() {
        let html = "<html><body><div style='display: none;'>Hidden</div></body></html>";

        let formatter = Formatter::new();
        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();
        assert!(list.is_empty());
    }

    #[test]
    fn test_to_display_with_breaks_and_deletions() {
        let html = r#"
        <html>
            <body>
                <div style="display: none;">Hidden</div>
                <p>First paragraph.</p>
                <span class="type1">Should be removed</span>
                <p>Second paragraph.</p>
                <span class="type2">Should have extra spacing</span>
                <p>Third paragraph.</p>
            </body>
        </html>
    "#;

        let formatter = Formatter::test(FormatConfig::new(
            HashSet::from(["span.type1".to_string()]),
            HashSet::from(["span.type2".to_string()]),
            HashMap::new(),
        ));

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![
                Span::from("First paragraph."),
                Span::from(" "),
                Span::from("Second paragraph."),
            ]),
            Line::default(),
            Line::from(Span::from("Should have extra spacing")),
            Line::default(),
            Line::from("Third paragraph."),
        ];

        assert_eq!(list, expected_output);
    }

    #[test]
    fn test_to_display_with_breaks_and_deletions_class_only() {
        let html = r#"
        <html>
            <body>
                <div style="display: none;">Hidden</div>
                <p>First paragraph.</p>
                <span class="type1">Should be removed</span>
                <p>Second paragraph.</p>
                <span class="type2">Should have extra spacing</span>
                <p class="type1">Third paragraph.</p>
            </body>
        </html>
    "#;

        let formatter = Formatter::test(FormatConfig::new(
            HashSet::from([".type1".to_string()]),
            HashSet::from([".type2".to_string()]),
            HashMap::new(),
        ));

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![
                Span::from("First paragraph."),
                Span::from(" "),
                Span::from("Second paragraph."),
            ]),
            Line::default(),
            Line::from(Span::from("Should have extra spacing")),
            Line::default(),
        ];

        assert_eq!(list, expected_output);
    }

    #[test]
    fn test_to_display_id_based_formatting() {
        let html = r#"
        <html>
            <body>
                <div style="display: none;">Hidden</div>
                <p>First paragraph.</p>
                <div id="remove-me">This should be removed</div>
                <p>Second paragraph.</p>
                <div id="extra-space">This should have extra spacing</div>
                <p>Third paragraph.</p>
            </body>
        </html>
    "#;

        let formatter = Formatter::test(FormatConfig::new(
            HashSet::from(["#remove-me".to_string()]),
            HashSet::from(["#extra-space".to_string()]),
            HashMap::new(),
        ));

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![
                Span::from("First paragraph."),
                Span::from(" "),
                Span::from("Second paragraph."),
            ]),
            Line::default(),
            Line::from(Span::from("This should have extra spacing")),
            Line::default(),
            Line::from("Third paragraph."),
        ];

        assert_eq!(list, expected_output);
    }

    #[test]
    fn test_multi_line_preformatted_text_with_tabs_and_spans() {
        let html = r#"
            <pre>
This is line one.
    This is line two with a <b>bold</b> word.
        This is line three with an <i>italic</i> word.
            </pre>
        "#;

        let formatter = Formatter::new();
        let binding = Html::parse_document(html);
        let result = Text::from(
            binding
                .select(&Selector::parse("body").unwrap())
                .flat_map(|element| formatter.to_display(element))
                .collect::<Vec<Line>>(),
        );

        let expected = Text::from(vec![
            Line::default(),
            Line::from("This is line one."),
            Line::from_iter([
                Span::from("    This is line two with a "),
                Span::styled("bold", Style::default().add_modifier(Modifier::BOLD)),
                Span::from(" word."),
            ]),
            Line::from_iter([
                Span::from("        This is line three with an "),
                Span::styled("italic", Style::default().add_modifier(Modifier::ITALIC)),
                Span::from(" word."),
            ]),
            Line::default(),
        ]);

        assert_eq!(result, expected);
    }
}
