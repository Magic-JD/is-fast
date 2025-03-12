use crate::cli::command::ColorMode;
use crate::config::load::Config;
use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use crate::search_engine::link::Link;
use crate::search_engine::scrape::scrape;
use crate::transform::filter::filter;
use crate::transform::format::to_display;
use nu_ansi_term::{Color, Style};
use ratatui::prelude::Text;
use ratatui::style::Style as RatStyle;
use ratatui::style::{Color as RatColor, Modifier};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use scraper::Html;
use std::fs;
use std::sync::Arc;

type ToHtml = Arc<dyn Fn(&Link) -> Result<String, IsError> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct PageExtractor {
    pub convert_to_html: ToHtml,
    pub color_mode: ColorMode,
    pub selector: Option<String>,
    pub element_separator: Option<char>,
}

impl PageExtractor {
    pub fn from_url(
        color_mode: ColorMode,
        selector: Option<String>,
        element_separator: Option<char>,
    ) -> Self {
        Self {
            convert_to_html: Arc::new(|link| scrape(&link.url)),
            color_mode,
            selector,
            element_separator,
        }
    }

    pub fn from_file(
        color_mode: ColorMode,
        selector: Option<String>,
        element_separator: Option<char>,
    ) -> Self {
        Self {
            convert_to_html: Arc::new(|link| {
                fs::read_to_string(&link.title).map_err(|e| General(e.to_string()))
            }),
            color_mode,
            selector,
            element_separator,
        }
    }

    pub fn get_paragraph(&self, link: &Link) -> Paragraph<'static> {
        let text = self.get_tui_text(link);
        match self.color_mode {
            ColorMode::Never => Paragraph::new(
                text.lines
                    .iter()
                    .map(ToString::to_string)
                    .map(Line::from)
                    .collect::<Vec<Line>>(),
            ),
            _ => Paragraph::new(text),
        }
    }

    fn get_tui_text(&self, link: &Link) -> Text<'static> {
        (self.convert_to_html)(link)
            .map(|html| PageExtractor::sanitize(&html))
            .map(|res| Html::parse_document(res.as_str()))
            .and_then(|html| {
                filter(
                    &html,
                    self.selector
                        .clone()
                        .unwrap_or_else(|| Config::get_selectors(&link.url)),
                )
                .and_then(|elements| to_display(elements, self.element_separator))
            })
            .unwrap_or_else(|_| Text::from("Failed to convert to text"))
    }

    pub fn get_text(&self, link: &Link) -> String {
        self.get_tui_text(link)
            .lines
            .into_iter()
            .map(|line| match self.color_mode {
                ColorMode::Always => Self::convert_rat_to_ansi(line),
                _ => line.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn convert_rat_to_ansi(line: Line) -> String {
        let mut painted = String::new();
        for span in line.spans {
            painted.push_str(&Self::apply_to_text(span.content.as_ref(), span.style));
        }

        Self::apply_to_text(&painted, line.style)
    }

    fn apply_to_text(content: &str, rat_style: RatStyle) -> String {
        let mut style = Style::new();
        if let Some(rat_color) = rat_style.fg {
            let color = match rat_color {
                RatColor::Black => Color::Black,
                RatColor::Red => Color::Red,
                RatColor::Green => Color::Green,
                RatColor::Yellow => Color::Yellow,
                RatColor::Blue => Color::Blue,
                RatColor::Magenta => Color::Magenta,
                RatColor::Cyan => Color::Cyan,
                RatColor::White => Color::White,
                RatColor::Gray => Color::LightGray,
                RatColor::DarkGray => Color::DarkGray,
                RatColor::LightRed => Color::LightRed,
                RatColor::LightGreen => Color::LightGreen,
                RatColor::LightYellow => Color::LightYellow,
                RatColor::LightBlue => Color::LightBlue,
                RatColor::LightMagenta => Color::LightMagenta,
                RatColor::LightCyan => Color::LightCyan,
                RatColor::Rgb(r, g, b) => Color::Rgb(r, g, b),
                _ => Color::Default,
            };
            style = style.fg(color);
        }
        for modifier in rat_style.add_modifier.iter() {
            style = match modifier {
                Modifier::BOLD => style.bold(),
                Modifier::ITALIC => style.italic(),
                Modifier::UNDERLINED => style.underline(),
                Modifier::DIM => style.dimmed(),
                Modifier::REVERSED => style.reverse(),
                Modifier::CROSSED_OUT => style.strikethrough(),
                _ => style,
            };
        }
        let paint = style.paint(content);
        format!("{}", paint)
    }

    fn sanitize(html: &str) -> String {
        html.replace('\t', "    ").replace(['\r', '\u{feff}'], "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;
    use std::path::Path;

    #[test]
    fn test_ansi_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let link = Link::new(path_sample, String::default());

        let ansi_text = PageExtractor::from_file(ColorMode::Always, None, None)
            .get_text(&link)
            .to_owned();

        let path_output = Path::new("tests/data/expected_ansi_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        assert_eq!(ansi_text, expected_content);
    }

    #[test]
    fn test_plain_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let link = Link::new(path_sample, String::default());

        let plain_text = PageExtractor::from_file(ColorMode::Never, None, None)
            .get_text(&link)
            .to_owned();

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        assert_eq!(plain_text, expected_content);
    }

    #[test]
    fn test_tui_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let link = Link::new(path_sample, String::default());

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        let result = PageExtractor::from_file(ColorMode::Always, None, None).get_tui_text(&link);

        let expected_lines: Vec<_> = expected_content.lines().collect();
        let binding = result.to_string();
        let result_lines: Vec<_> = binding.lines().collect();

        let min_len = expected_lines.len().min(result_lines.len());

        for i in 0..min_len {
            if expected_lines[i] != result_lines[i] {
                // Additional newline on from text input
                panic!(
                    "Mismatch at line {}:\nExpected: {:?}\nGot: {:?}",
                    i + 1,
                    expected_lines[i],
                    result_lines[i]
                );
            }
        }

        let length = result
            .lines
            .iter()
            .flat_map(|line| line.spans.clone())
            .collect::<Vec<Span>>()
            .len();
        assert_eq!(length, 559);
    }
}
