use crate::config::load::Config;
use once_cell::sync::Lazy;
use ratatui::prelude::{Color, Line, Span, Style};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Style as SyntectStyle, Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

static DEFAULT_LANGUAGE: Lazy<String> = Lazy::new(Config::get_default_language);
static SYNTAX_HIGHLIGHTING_THEME: Lazy<String> = Lazy::new(Config::get_syntax_highlighting_theme);
static THEME_SET: Lazy<ThemeSet> = Lazy::new(ThemeSet::load_defaults);
static DEFAULT_THEME: Lazy<Theme> = Lazy::new(Theme::default);
static SYNTAX_SET: Lazy<SyntaxSet> = Lazy::new(SyntaxSet::load_defaults_newlines);

pub fn highlight_code(text: String, language: &str) -> Vec<Line<'static>> {
    let syntax = SYNTAX_SET
        .find_syntax_by_token(language) // Attempt to use language from css
        .or_else(|| SYNTAX_SET.find_syntax_by_token(DEFAULT_LANGUAGE.as_str())) // Attempt to use language from config
        .unwrap_or_else(|| SYNTAX_SET.find_syntax_plain_text()); // Use plain text

    let theme_set = &THEME_SET;
    let default_theme = &DEFAULT_THEME;
    let theme = theme_set
        .themes
        .get(SYNTAX_HIGHLIGHTING_THEME.as_str())
        .unwrap_or_else(|| default_theme);

    let mut highlighter = HighlightLines::new(syntax, theme);

    LinesWithEndings::from(&text)
        .map(|line| highlight_line(&SYNTAX_SET, &mut highlighter, line))
        .collect()
}

fn highlight_line(
    syntax_set: &SyntaxSet,
    highlighter: &mut HighlightLines,
    line: &str,
) -> Line<'static> {
    let highlighted = highlighter.highlight_line(line, syntax_set).unwrap();
    let styled_spans = highlighted
        .iter()
        .map(|(style, content)| Span::styled(content.to_string(), convert_syntect_style(*style)))
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
