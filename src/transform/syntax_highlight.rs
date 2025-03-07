use crate::config::load::Config;
use once_cell::sync::Lazy;
use ratatui::prelude::{Color, Line, Span, Style};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static DEFAULT_LANGUAGE: Lazy<&String> = Lazy::new(Config::get_default_language);
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static DEFAULT_SYNTAX: Lazy<&'static SyntaxReference> = Lazy::new(|| {
    let default_lang = DEFAULT_LANGUAGE.as_str();
    SYNTAX_SET
        .find_syntax_by_token(default_lang) // Use language from config
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text()) // Fallback to plain text
});
static DEFAULT_THEME: Lazy<&'static Theme> = Lazy::new(|| {
    THEME_SET
        .themes
        .get(&Config::get_syntax_highlighting_theme())
        .unwrap_or_else(|| {
            THEME_SET
                .themes
                .values()
                .next()
                .expect("No themes available")
        })
});

pub fn highlight_code(text: &str, language: &str) -> Vec<Line<'static>> {
    let syntax = SYNTAX_SET
        .find_syntax_by_token(language) // Attempt to use language from css
        .unwrap_or_else(|| *DEFAULT_SYNTAX);

    let mut highlighter = HighlightLines::new(syntax, *DEFAULT_THEME);
    LinesWithEndings::from(text)
        .map(|line| highlight_line(&SYNTAX_SET, &mut highlighter, line))
        .collect()
}

fn highlight_line(
    syntax_set: &SyntaxSet,
    highlighter: &mut HighlightLines,
    line: &str,
) -> Line<'static> {
    let highlighted_string = highlighter
        .highlight_line(line, syntax_set)
        .expect("Line could not be highlighted."); // Should not happen -> if it does it's important to fix
    let styled_spans = highlighted_string
        .iter()
        .map(|(style, content)| Span::styled((*content).to_string(), convert_syntect_style(*style)))
        .filter(|s| s.content != "\n")
        .collect::<Vec<Span>>();
    Line::from(styled_spans)
}

fn convert_syntect_style(syntect_style: SyntectStyle) -> Style {
    Style::default().fg(Color::Rgb(
        syntect_style.foreground.r,
        syntect_style.foreground.g,
        syntect_style.foreground.b,
    ))
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_selection() {
        assert!(
            !THEME_SET.themes.is_empty(),
            "Theme set should not be empty"
        );
        assert!(
            THEME_SET
                .themes
                .values()
                .any(|theme| theme == *DEFAULT_THEME),
            "Selected theme should be present in the theme set"
        );
    }

    #[test]
    fn test_highlight_line() {
        let syntax = SYNTAX_SET.find_syntax_plain_text();
        let mut highlighter = HighlightLines::new(syntax, *DEFAULT_THEME);
        let result = highlight_line(&SYNTAX_SET, &mut highlighter, "fn main() {}");

        assert!(
            !result.spans.is_empty(),
            "Highlighted line should not be empty"
        );
        assert_eq!(result.to_string(), "fn main() {}");
    }

    #[test]
    fn test_highlight_code_rust() {
        let code = r#"fn main() {
    println!("Hello, world!");
}"#
        .to_string();
        let highlighted = highlight_code(&code, "rust");
        let expected = vec![
            Line::from_iter([
                Span::styled("fn", Style::default().fg(Color::Rgb(180, 142, 173))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("main", Style::default().fg(Color::Rgb(143, 161, 179))),
                Span::styled("(", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(")", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(" ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("{", Style::default().fg(Color::Rgb(192, 197, 206))),
            ]),
            Line::from_iter([
                Span::styled("    ", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("println!", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("(", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled("\"", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(
                    "Hello, world!",
                    Style::default().fg(Color::Rgb(163, 190, 140)),
                ),
                Span::styled("\"", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(")", Style::default().fg(Color::Rgb(192, 197, 206))),
                Span::styled(";", Style::default().fg(Color::Rgb(192, 197, 206))),
            ]),
            Line::from_iter([Span::styled(
                "}",
                Style::default().fg(Color::Rgb(192, 197, 206)),
            )]),
        ];

        assert_eq!(highlighted, expected);
    }
}
