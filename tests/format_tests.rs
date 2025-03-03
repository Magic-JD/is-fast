use is_fast::formatting::format::to_display;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use ratatui::style::{Color, Modifier, Style, Styled};
    use super::*;
    use ratatui::text::{Line, Span, Text};
    use is_fast::scrapers::scrape::sanitize;

    #[test]
    fn test_to_display_simple_html() {
        let url = "http://example.com";
        let html = r#"
            <html>
                <head><title>Test</title></head>
                <body>
                    <h1>Hello, World!</h1>
                    <p>This is a <strong>test</strong>.</p>
                </body>
            </html>
        "#;

        let result = to_display(url, html).expect("Expected valid parsed output");

        let expected = Text::from(vec![
            Line::from(Span::styled(
                "Hello, World!",
                Style::default().add_modifier(Modifier::BOLD),
            )).set_style(Style::default().add_modifier(Modifier::BOLD)),
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
        let url = "http://example.com";
        let html = r#"
            <html>
                <body>
                    <pre><code class="language-rust">fn main() { println!("Hello, Rust!"); }</code></pre>
                </body>
            </html>
        "#;

        let result = to_display(url, html).expect("Expected valid parsed output");

        // Since `highlight_code` transforms the code, we check if the output contains expected text
        assert!(result.to_string().contains("fn main() { println!(\"Hello, Rust!\"); }"));


        let expected = Text::from(vec![
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
                Span::styled("Hello, Rust!", Style::default().fg(Color::Rgb(163, 190, 140))),
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
        let url = "http://example.com";
        let html = "<html><body><div style='display: none;'>Hidden</div></body></html>";

        let result = to_display(url, html);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "No content found");
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

        let result = to_display("http://example.com", html).unwrap();

        let expected = Text::from(vec![
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


    #[test]
    fn test_large_scale_file_as_expected() {
        let path_sample = Path::new("tests/data/sample.html");
        let dirty = fs::read_to_string(path_sample).expect("Failed to read test HTML file");
        let html = sanitize(&dirty);

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output).expect("Failed to read test HTML file");

        let result = to_display("http://example.com", &html).unwrap();

        let content = result.to_string();
        assert_eq!(content, expected_content);

        let length = content.len();
        assert_eq!(length, 10203);
    }
}
