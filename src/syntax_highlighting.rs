use ratatui::prelude::{Color, Line, Span, Style};
use syntect::easy::HighlightLines;
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;
use syntect::highlighting::{Style as SyntectStyle, ThemeSet};

pub fn highlight_code(text: &str, language: &str) -> Vec<Line<'static>> {
    let syntax_set = SyntaxSet::load_defaults_newlines();
    let theme_set = ThemeSet::load_defaults();
    let syntax = syntax_set
        .find_syntax_by_token(language)
        .unwrap_or(syntax_set.find_syntax_by_token("java").unwrap()); // Default to java - should be able to set in settings.
    let mut highlighter = HighlightLines::new(syntax, &theme_set.themes["base16-ocean.dark"]); // Trial different themes?

    let mut lines = Vec::new();

    for line in LinesWithEndings::from(text) {
        let highlighted = highlighter.highlight_line(line, &syntax_set).unwrap();

        let styled_spans = highlighted
            .iter()
            .map(|(style, content)| {
                Span::styled(content.to_string(), convert_syntect_style(*style))
            })
            .collect::<Vec<Span>>();

        lines.push(Line::from(styled_spans));
    }

    lines
}

fn convert_syntect_style(syntect_style: SyntectStyle) -> Style {
    Style::default().fg(Color::Rgb(
        syntect_style.foreground.r,
        syntect_style.foreground.g,
        syntect_style.foreground.b,
    ))
}
