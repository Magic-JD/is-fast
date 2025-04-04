use crate::config::color_conversion::{Color, Style};
use crate::config::site::SyntaxConfig;
use crate::page::structure::{Line, Span};
use once_cell::sync::Lazy;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::{SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static BACKUP_THEME: Lazy<Theme> = Lazy::new(Theme::default);

pub struct SyntaxHighlighter {
    config: SyntaxConfig,
}

impl SyntaxHighlighter {
    pub fn new(config: SyntaxConfig) -> Self {
        Self { config }
    }

    pub fn highlight_code(&self, text: &str, language: &str) -> Vec<Line> {
        let syntax = SYNTAX_SET
            .find_syntax_by_token(language) // Attempt to use language from css
            .unwrap_or_else(|| self.get_default_syntax());

        let mut highlighter = HighlightLines::new(syntax, self.get_default_theme());
        LinesWithEndings::from(text)
            .map(|line| Self::highlight_line(&SYNTAX_SET, &mut highlighter, line))
            .collect()
    }

    fn highlight_line(
        syntax_set: &SyntaxSet,
        highlighter: &mut HighlightLines,
        line: &str,
    ) -> Line {
        let highlighted_string = highlighter
            .highlight_line(line, syntax_set)
            .expect("Line could not be highlighted."); // Should not happen -> if it does it's important to fix
        let styled_spans = highlighted_string
            .iter()
            .map(|(style, content)| Span::styled(content, Self::convert_syntect_style(*style)))
            .filter(|s| !s.content.is_empty())
            .collect::<Vec<Span>>();
        Line::from(styled_spans)
    }

    fn get_default_theme(&self) -> &Theme {
        THEME_SET
            .themes
            .get(self.config.get_syntax_highlighting_theme())
            .unwrap_or_else(|| THEME_SET.themes.values().next().unwrap_or(&BACKUP_THEME))
    }

    fn get_default_syntax(&self) -> &SyntaxReference {
        let default_lang = self.config.get_syntax_default_language();
        SYNTAX_SET
            .find_syntax_by_token(default_lang) // Use language from config
            .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text()) // Fallback to plain text
    }

    fn convert_syntect_style(syntect_style: SyntectStyle) -> Style {
        Style::fg(Color::from_syntect_color(syntect_style.foreground))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SYNTAX_CONFIG: Lazy<SyntaxConfig> = Lazy::new(|| SyntaxConfig {
        syntax_default_language: String::from("rust"),
        syntax_highlighting_theme: String::from("base16-ocean.dark"),
    });

    #[test]
    fn test_highlight_line() {
        let syntax_highlighter = SyntaxHighlighter::new(SYNTAX_CONFIG.clone());
        let result = syntax_highlighter.highlight_code("fn main() {}", "rust");

        assert!(
            !result.first().unwrap().spans.is_empty(),
            "Highlighted line should not be empty"
        );
        assert_eq!(result.first().unwrap().content(), "fn main() {}");
    }

    #[test]
    fn test_highlight_code_rust() {
        let code = r#"fn main() {
    println!("Hello, world!");
}"#
        .to_string();
        let syntax_highlighter = SyntaxHighlighter::new(SYNTAX_CONFIG.clone());
        let highlighted = syntax_highlighter.highlight_code(&code, "rust");
        let expected = vec![
            Line::from(vec![
                Span::styled("fn", Style::fg(Color::rgb(180, 142, 173))),
                Span::styled(" ", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("main", Style::fg(Color::rgb(143, 161, 179))),
                Span::styled("(", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled(")", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled(" ", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("{", Style::fg(Color::rgb(192, 197, 206))),
            ]),
            Line::from(vec![
                Span::styled("    ", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("println!", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("(", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("\"", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled("Hello, world!", Style::fg(Color::rgb(163, 190, 140))),
                Span::styled("\"", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled(")", Style::fg(Color::rgb(192, 197, 206))),
                Span::styled(";", Style::fg(Color::rgb(192, 197, 206))),
            ]),
            Line::from(vec![Span::styled(
                "}",
                Style::fg(Color::rgb(192, 197, 206)),
            )]),
        ];

        assert_eq!(highlighted, expected);
    }
}
