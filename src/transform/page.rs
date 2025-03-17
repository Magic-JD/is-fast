use crate::cli::command::ColorMode;
use crate::config::load::{Config, ExtractionConfig};
use crate::errors::error::IsError;
use crate::errors::error::IsError::{Io, Scrape};
use crate::search_engine::link::HtmlSource;
use crate::search_engine::scrape;
use crate::search_engine::scrape::scrape;
use crate::transform::filter::filter;
use crate::transform::format::Formatter;
use nu_ansi_term::{Color, Style};
use once_cell::sync::Lazy;
use ratatui::prelude::Text;
use ratatui::style::Style as RatStyle;
use ratatui::style::{Color as RatColor, Modifier};
use ratatui::text::Line;
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Selector};
use std::fs;

static FORMATTER: Lazy<Formatter> = Lazy::new(Formatter::new);

#[derive(Clone)]
pub struct PageExtractor {
    config: ExtractionConfig,
}

impl PageExtractor {
    pub fn new() -> Self {
        Self {
            config: Config::get_extractor_config(),
        }
    }

    pub fn config(&self) -> &ExtractionConfig {
        &self.config
    }

    pub fn get_paragraph(&self, link: &HtmlSource) -> (String, Paragraph<'static>) {
        let (title, text) = self.get_tui_text(link);
        let paragraph = match self.config().color_mode() {
            ColorMode::Never => Paragraph::new(
                text.lines
                    .iter()
                    .map(ToString::to_string)
                    .map(Line::from)
                    .collect::<Vec<Line>>(),
            ),
            _ => Paragraph::new(text),
        };
        (title, paragraph)
    }

    fn get_tui_text(&self, html_source: &HtmlSource) -> (String, Text<'static>) {
        let html_result: Result<String, IsError> = match html_source {
            HtmlSource::LinkSource(link) => scrape(&link.url),
            HtmlSource::FileSource(file) => fs::read_to_string(&file.file_path).map_err(Io),
        };
        let selector = Selector::parse("title").expect("invalid title selector");
        log::debug!("Preparing to parse HTML");
        let html = html_result
            .map(|html| PageExtractor::sanitize(&html))
            .map(|sanitized| Html::parse_document(&sanitized));
        log::debug!("HTML parsed");

        html.and_then(|html| {
            let title = Self::extract_title(&selector, &html);
            let text = self.extract_text(html_source, &html)?;
            Ok((title, text))
        })
        .unwrap_or_else(|err| {
            if let HtmlSource::LinkSource(link) = html_source {
                scrape::cache_purge(&link.url);
            };
            (
                String::from("Failed to retrieve"),
                Text::from(err.to_string()),
            )
        })
    }

    fn extract_text(
        &self,
        html_source: &HtmlSource,
        html: &Html,
    ) -> Result<Text<'static>, IsError> {
        filter(
            html,
            self.config().get_selectors(html_source.get_url()),
        )
            .map(|elements| self.process_elements(elements))
            .and_then(|text| {
                if text
                    .lines
                    .iter()
                    .any(|line| !line.to_string().trim().is_empty())
                {
                    Ok(text)
                } else {
                    Err(Scrape(String::from("Result returned, but not text found. Either the expected html was not retrieved, or the selectors are incorrectly configured.")))
                }
            })
    }

    fn extract_title(selector: &Selector, html: &Html) -> String {
        let title = html.select(selector).next().map_or_else(
            || {
                log::error!("No title found for page ");
                "Unknown Title".to_string()
            },
            |t| t.text().collect::<String>(),
        );
        log::debug!("Title extracted: {}", title);
        title
    }

    fn process_elements(&self, elements: Vec<ElementRef>) -> Text<'static> {
        log::trace!("Processing all elements");
        let mut lines: Vec<Vec<Line>> = elements
            .into_iter()
            .map(|element| FORMATTER.to_display(element))
            .filter(|lines| !lines.is_empty())
            .collect();
        let nth_element = self.config().nth_element();
        if !nth_element.is_empty() {
            lines = lines
                .into_iter()
                .enumerate()
                .filter_map(|(index, text_block)| {
                    if nth_element.contains(&(index + 1)) {
                        Some(text_block)
                    } else {
                        None
                    }
                })
                .collect::<Vec<Vec<Line>>>();
        }
        Text::from(lines.into_iter().flatten().collect::<Vec<Line>>())
    }

    pub fn get_text(&self, html_source: &HtmlSource) -> (String, String) {
        let (title, text) = self.get_tui_text(html_source);
        let plaintext = text
            .lines
            .into_iter()
            .map(|line| match self.config().color_mode() {
                ColorMode::Always => Self::convert_rat_to_ansi(line),
                _ => line.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n");
        (title, plaintext)
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
        format!("{paint}")
    }

    fn sanitize(html: &str) -> String {
        html.replace('\t', "    ").replace(['\r', '\u{feff}'], "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_engine::link::File;
    use crate::search_engine::link::HtmlSource::FileSource;
    use globset::GlobSet;
    use ratatui::text::Span;
    use std::collections::HashMap;
    use std::path::Path;

    impl PageExtractor {
        pub fn test_init(config: ExtractionConfig) -> Self {
            Self { config }
        }
    }

    #[test]
    fn test_ansi_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let file = File::new(path_sample, String::new());

        let config = ExtractionConfig::new(
            ColorMode::Always,
            vec![],
            HashMap::new(),
            Some("body".to_string()),
            GlobSet::empty(),
            vec![],
        );
        let (filename, ansi_text) = PageExtractor::test_init(config)
            .get_text(&FileSource(file))
            .to_owned();

        let path_output = Path::new("tests/data/expected_ansi_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();
        let expected_filename = String::from("for and range - Rust By Example");
        assert_eq!(filename, expected_filename);
        assert_eq!(ansi_text, expected_content);
    }

    #[test]
    fn test_plain_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let file = File::new(path_sample, String::new());

        let config = ExtractionConfig::new(
            ColorMode::Tui,
            vec![],
            HashMap::new(),
            Some("body".to_string()),
            GlobSet::empty(),
            vec![],
        );
        let (filename, plain_text) = PageExtractor::test_init(config)
            .get_text(&FileSource(file))
            .to_owned();

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        let expected_filename = String::from("for and range - Rust By Example");
        assert_eq!(filename, expected_filename);
        assert_eq!(plain_text, expected_content);
    }

    #[test]
    fn test_restrictive_selectors() {
        let path_sample = String::from("tests/data/sample.html");
        let file = File::new(path_sample, String::new());

        let config = ExtractionConfig::new(
            ColorMode::Tui,
            vec![],
            HashMap::new(),
            Some("p".to_string()),
            GlobSet::empty(),
            vec![],
        );
        let (filename, plain_text) = PageExtractor::test_init(config)
            .get_text(&FileSource(file))
            .to_owned();

        let path_output = Path::new("tests/data/expected_selected.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        let expected_filename = String::from("for and range - Rust By Example");
        assert_eq!(filename, expected_filename);
        assert_eq!(plain_text, expected_content);
    }

    #[test]
    fn test_restrictive_selectors_nth() {
        let path_sample = String::from("tests/data/sample.html");
        let file = File::new(path_sample, String::new());

        let config = ExtractionConfig::new(
            ColorMode::Tui,
            vec![1, 3],
            HashMap::new(),
            Some("p".to_string()),
            GlobSet::empty(),
            vec![],
        );
        let (filename, plain_text) = PageExtractor::test_init(config)
            .get_text(&FileSource(file))
            .to_owned();

        let path_output = Path::new("tests/data/expected_nth.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        let expected_filename = String::from("for and range - Rust By Example");
        assert_eq!(filename, expected_filename);
        assert_eq!(plain_text, expected_content);
    }

    #[test]
    fn test_tui_text_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let file = File::new(path_sample, String::new());
        let source = FileSource(file);

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read expected text file")
            .to_owned();

        let config = ExtractionConfig::new(
            ColorMode::Tui,
            vec![],
            HashMap::new(),
            Some("body".to_string()),
            GlobSet::empty(),
            vec![],
        );
        let (_, text) = PageExtractor::test_init(config).get_tui_text(&source);

        let expected_lines: Vec<_> = expected_content.lines().collect();
        let binding = text.to_string();
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

        let length = text
            .lines
            .iter()
            .flat_map(|line| line.spans.clone())
            .collect::<Vec<Span>>()
            .len();
        assert_eq!(length, 559);
    }
}
