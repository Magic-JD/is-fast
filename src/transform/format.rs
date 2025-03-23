use crate::config::color_conversion::Style;
use crate::config::format::FormatConfig;
use crate::page::structure::{Line, Span};
use crate::transform::syntax_highlight::SyntaxHighlighter;
use scraper::{Element, ElementRef, Node};

pub struct Formatter {
    config: FormatConfig,
    syntax_highlighter: SyntaxHighlighter,
}

impl Formatter {
    pub fn new(config: FormatConfig, syntax_highlighter: SyntaxHighlighter) -> Formatter {
        Formatter {
            config,
            syntax_highlighter,
        }
    }

    pub fn to_display(&self, element: ElementRef) -> Vec<Line> {
        log::trace!("Converting element to display lines: {:?}", element);
        let mut lines = self
            .to_lines(element, element.value().name() == "pre")
            .into_iter()
            .map(standardize_empty)
            .map(Line::flatten)
            .collect::<Vec<Line>>();
        lines.dedup();
        lines
    }

    fn to_lines(&self, element: ElementRef, pre_formatted: bool) -> Vec<Line> {
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
                Line::default(),
            ];
        }
        let style = self.config.style_for_tag(&element);
        if tag_name == "code" {
            // Handle code differently due to performance issues.
            let language_type = extract_language_type(element);
            let code_text = extract_code(element);
            let mut lines = self
                .syntax_highlighter
                .highlight_code(&code_text, &language_type);
            if let Some(style) = style {
                lines = lines
                    .into_iter()
                    .map(|line| line.set_style(style))
                    .collect();
            }
            return lines;
        }

        let mut lines = Vec::new();

        if tag_name == "img" {
            // Show there is an image without rendering the image.
            lines.push(create_optionally_styled_line("IMAGE", style.as_ref()));
        } else {
            lines = self.extract_lines(element, pre_formatted || tag_name == "pre", style.as_ref());
        }

        if lines.is_empty() {
            return vec![];
        }

        if tag_name == "li" {
            lines = handle_list_item(&element, style.as_ref(), lines);
        }

        if self.config.is_block_element(&element) {
            if let Some(styled) = style {
                lines = lines
                    .into_iter()
                    .map(|line| line.set_style(styled))
                    .collect();
            }
            lines.insert(0, Line::default());
            lines.push(Line::default());
        }
        // Indent if needed.
        if self.config.is_indent_element(&element) {
            let indent_block = "  ";
            for line in &mut lines {
                if let Some(span) = line.spans.first_mut() {
                    span.content = format!("{indent_block}{}", span.content);
                }
            }
        }
        lines
    }

    fn extract_lines(
        &self,
        element: ElementRef,
        pre_formatted: bool,
        style: Option<&Style>,
    ) -> Vec<Line> {
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
                let mut element_lines = self.to_lines(*element, pre_formatted);
                if let Some(style) = style {
                    element_lines = element_lines
                        .into_iter()
                        .map(|line| line.set_style(*style))
                        .collect();
                }
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
    if line.content().trim().is_empty() {
        Line::default()
    } else {
        line
    }
}

fn create_optionally_styled_line(content: &str, style: Option<&Style>) -> Line {
    if let Some(style) = style {
        Line::from_single(Span::styled(content, *style))
    } else {
        Line::from_single(Span::from(content))
    }
}

fn merge_with_previous_line(lines: &mut Vec<Line>, mut new_lines: Vec<Line>) {
    if let (Some(prev_end), Some(new_start)) = (lines.last_mut(), new_lines.first_mut()) {
        if let (Some(end), Some(start)) = (prev_end.spans.last(), new_start.spans.first()) {
            // Text from two different elements should almost always have a space - only exception for punctuation.
            // However often that space is achieved through css rather than the text.
            // Therefore, we check this manually.
            if !(start.content.is_empty()
                || end.content.is_empty()
                || end.content.ends_with(' ')
                || start.content.starts_with(' ')
                || matches!(start.content.chars().next(), Some(c) if ".:;,<[{()}]>\"\\`'/".contains(c))
                || matches!(end.content.chars().last(), Some(c) if "<[{()}]>`'\"\\/".contains(c)))
            {
                prev_end.spans.push(Span::from(" "));
            }
            prev_end.spans.append(&mut new_start.spans);
            new_lines.drain(..1);
        }
    }

    lines.append(&mut new_lines);
}

fn handle_list_item(
    element: &ElementRef,
    style: Option<&Style>,
    mut lines: Vec<Line>,
) -> Vec<Line> {
    let marker = determine_marker(element);
    lines.retain(|line| !line.spans.is_empty());
    if let Some(line) = lines.first_mut() {
        if let Some(style) = style {
            line.spans.insert(0, Span::styled(&marker, *style));
        } else {
            line.spans.insert(0, Span::from(&marker));
        }
    }
    lines
}

fn determine_marker(element: &ElementRef) -> String {
    if let Some(value) = element.value().attr("value") {
        return format!("{value}. ");
    }
    element
        .parent_element()
        .filter(|parent| parent.value().name() == "ol")
        .and_then(|parent| {
            find_child_index(parent, element).map(|idx| {
                idx + parent
                    .value()
                    .attr("start")
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(1)
            })
        })
        .map_or_else(|| "• ".into(), |idx| format!("{idx}. "))
}

fn find_child_index(parent: ElementRef, child: &ElementRef) -> Option<usize> {
    parent
        .children()
        .filter_map(ElementRef::wrap)
        .enumerate()
        .find_map(|(index, el)| if el == *child { Some(index) } else { None })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::color_conversion::Color;
    use crate::config::site::SyntaxConfig;
    use scraper::{Html, Selector};
    use std::collections::{HashMap, HashSet};

    fn bold() -> Style {
        let mut bold_style = Style::default();
        bold_style.bold = Some(true);
        bold_style
    }

    fn italic() -> Style {
        let mut italic_style = Style::default();
        italic_style.italic = Some(true);
        italic_style
    }

    fn basic_format_config() -> FormatConfig {
        let ignored_tages = HashSet::from(["head"])
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let block_elements = HashSet::from(["h1", "pre", "body"])
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let indent_elements = HashSet::from(["li"])
            .iter()
            .map(|s| s.to_string())
            .collect::<HashSet<_>>();
        let mut style_elements = HashMap::new();
        style_elements.insert("strong".to_string(), bold());
        style_elements.insert("i".to_string(), italic());
        style_elements.insert("h1".to_string(), bold());
        style_elements.insert("b".to_string(), bold());
        FormatConfig::new(
            ignored_tages,
            block_elements,
            indent_elements,
            style_elements,
        )
    }

    fn basic_syntax_highlighter() -> SyntaxHighlighter {
        SyntaxHighlighter::new(SyntaxConfig::default())
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

        let formatter = Formatter::new(basic_format_config(), basic_syntax_highlighter());
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected = vec![
            Line::default(),
            Line::from_single(Span::styled("Hello, World!", bold())).set_style(bold()),
            Line::default(),
            Line::from(vec![
                Span::from("This is a "),
                Span::styled("test", bold()),
                Span::from("."),
            ]),
            Line::default(),
        ];

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

        let formatter = Formatter::new(
            basic_format_config(),
            SyntaxHighlighter::new(SyntaxConfig {
                syntax_default_language: "java".to_string(),
                syntax_highlighting_theme: "base16-ocean.dark".to_string(),
            }),
        );
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        // Since `highlight_code` transforms the code, we check if the output contains expected text
        assert!(result
            .iter()
            .map(Line::content)
            .collect::<Vec<String>>()
            .join("")
            .contains("fn main() { println!(\"Hello, Rust!\"); }"));

        let expected = vec![
            Line::default(),
            Line::from(vec![
                Span::styled("fn ", Style::fg(Color::rgb(180, 142, 173))),
                Span::styled("main", Style::fg(Color::rgb(143, 161, 179))),
                Span::styled("() { println!(\"", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("Hello, Rust!", Style::fg(Color::rgb(163, 190, 140))),
                Span::styled("\"); }", Style::fg(Color::rgb(192, 197, 206))),
            ]),
            Line::default(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_no_content() {
        let html = "<html><body><div style='display: none;'>Hidden</div></body></html>";

        let formatter = Formatter::new(basic_format_config(), basic_syntax_highlighter());
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

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::from(["span.type1".to_string()]),
                HashSet::from(["span.type2".to_string()]),
                HashSet::new(),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![Span::from("First paragraph. Second paragraph.")]),
            Line::default(),
            Line::from_single(Span::from("Should have extra spacing")),
            Line::default(),
            Line::from_single(Span::from("Third paragraph.")),
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

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::from([".type1".to_string()]),
                HashSet::from([".type2".to_string()]),
                HashSet::new(),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![Span::from("First paragraph. Second paragraph.")]),
            Line::default(),
            Line::from_single(Span::from("Should have extra spacing")),
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

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::from(["#remove-me".to_string()]),
                HashSet::from(["#extra-space".to_string()]),
                HashSet::new(),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );

        let binding = Html::parse_document(html);
        let list = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected_output = vec![
            Line::from(vec![Span::from("First paragraph. Second paragraph.")]),
            Line::default(),
            Line::from_single(Span::from("This should have extra spacing")),
            Line::default(),
            Line::from_single(Span::from("Third paragraph.")),
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

        let formatter = Formatter::new(basic_format_config(), basic_syntax_highlighter());
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected = vec![
            Line::default(),
            Line::from_single(Span::from("This is line one.")),
            Line::from(vec![
                Span::from("    This is line two with a "),
                Span::styled("bold", bold()),
                Span::from(" word."),
            ]),
            Line::from(vec![
                Span::from("        This is line three with an "),
                Span::styled("italic", italic()),
                Span::from(" word."),
            ]),
            Line::default(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_ordered_list() {
        let html = r#"
        <html>
            <body>
                <ol>
                    <li>First item</li>
                    <li>Second item</li>
                </ol>
                <ol start="5">
                    <li>Fifth item</li>
                    <li>Sixth item</li>
                </ol>
            </body>
        </html>
    "#;

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::new(),
                HashSet::from(["li".to_string()]),
                HashSet::new(),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected = vec![
            Line::default(),
            Line::from(vec![Span::from("1. First item")]),
            Line::default(),
            Line::from(vec![Span::from("2. Second item")]),
            Line::default(),
            Line::from(vec![Span::from("5. Fifth item")]),
            Line::default(),
            Line::from(vec![Span::from("6. Sixth item")]),
            Line::default(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_unordered_list() {
        let html = r#"
        <html>
            <body>
                <ul>
                    <li>Apple</li>
                    <li>Banana</li>
                    <li>Cherry</li>
                </ul>
            </body>
        </html>
    "#;

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::new(),
                HashSet::from(["li".to_string()]),
                HashSet::from(["li".to_string()]),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected = vec![
            Line::default(),
            Line::from(vec![Span::from("  • Apple")]),
            Line::default(),
            Line::from(vec![Span::from("  • Banana")]),
            Line::default(),
            Line::from(vec![Span::from("  • Cherry")]),
            Line::default(),
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn test_to_display_nested_ordered_list_with_values() {
        let html = r#"
    <html>
        <body>
            <ol>
                <li value="3"><span>The Condition is evaluated:</span>
                    <ol>
                        <li value="1"><span>If true, the control moves to Step 4.</span></li>
                        <li value="2"><span>If false, the control jumps to Step 7.</span></li>
                    </ol>
                </li>
                <li value="4"><span>The body of the loop is executed.</span></li>
            </ol>
        </body>
    </html>
    "#;

        let formatter = Formatter::new(
            FormatConfig::new(
                HashSet::new(),
                HashSet::from(["li".to_string()]),
                HashSet::from(["li".to_string()]),
                HashMap::new(),
            ),
            basic_syntax_highlighter(),
        );
        let binding = Html::parse_document(html);
        let result = binding
            .select(&Selector::parse("body").unwrap())
            .flat_map(|element| formatter.to_display(element))
            .collect::<Vec<Line>>();

        let expected = vec![
            Line::default(),
            Line::from(vec![Span::from("  3. The Condition is evaluated:")]),
            Line::from(vec![Span::from(
                "    1. If true, the control moves to Step 4.",
            )]),
            Line::from(vec![Span::from(
                "    2. If false, the control jumps to Step 7.",
            )]),
            Line::default(),
            Line::from(vec![Span::from("  4. The body of the loop is executed.")]),
            Line::default(),
        ];

        assert_eq!(result, expected);
    }
}
