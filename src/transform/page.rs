use crate::cli::command::ColorMode;
use crate::config::color_conversion::{Size, Style};
use crate::config::load::{Config, ExtractionConfig};
use crate::errors::error::IsError;
use crate::errors::error::IsError::{Io, Scrape};
use crate::page::structure::{Line, Span};
use crate::search_engine::link::HtmlSource;
use crate::search_engine::scrape;
use crate::search_engine::scrape::scrape;
use crate::transform::filter::filter;
use crate::transform::format::Formatter;
use crate::transform::syntax_highlight::SyntaxHighlighter;
use ratatui::text::{Line as RatLine, Text};
use ratatui::widgets::Paragraph;
use scraper::{ElementRef, Html, Selector};
use std::cmp::max;
use std::fs;

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

        let rat_lines: Vec<RatLine> = match self.config().color_mode() {
            ColorMode::Never => text.iter().map(Line::to_rat_colorless).collect(),
            _ => text.iter().map(Line::to_rat_colored).collect(),
        };

        let paragraph = Paragraph::new(Text::from(rat_lines));

        (title, paragraph)
    }

    fn get_tui_text(&self, html_source: &HtmlSource) -> (String, Vec<Line>) {
        let html_result: Result<String, IsError> = match html_source {
            HtmlSource::LinkSource(_) => scrape(html_source),
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
            if let HtmlSource::LinkSource(_) = html_source {
                scrape::cache_purge(html_source);
            }
            (
                String::from("Failed to retrieve"),
                vec![Line::from_single(Span::from(&err.to_string()))],
            )
        })
    }

    fn extract_text(&self, html_source: &HtmlSource, html: &Html) -> Result<Vec<Line>, IsError> {
        filter(
            html,
            self.config().get_selectors(html_source.get_url()),
        )
            .map(|elements| self.process_elements(html_source, elements))
            .and_then(|text| {
                if text
                    .iter()
                    .any(|line| !line.content().trim().is_empty())
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
        log::debug!("Title extracted: {title}");
        title
    }

    fn process_elements(&self, html_source: &HtmlSource, elements: Vec<ElementRef>) -> Vec<Line> {
        log::trace!("Processing all elements");
        let site_config = html_source.get_config();
        let format_config = site_config.get_format();
        let syntax_config = site_config.get_syntax();
        let mut lines: Vec<Vec<Line>> = elements
            .into_iter()
            .map(|element| {
                Formatter::new(
                    format_config.clone(),
                    SyntaxHighlighter::new(syntax_config.clone()),
                )
                .to_display(element)
            })
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
        lines.into_iter().flatten().collect::<Vec<Line>>()
    }

    pub fn get_text(&self, html_source: &HtmlSource) -> (String, String) {
        let (title, text) = self.get_tui_text(html_source);
        let plaintext = text
            .into_iter()
            .map(|line| match self.config().color_mode() {
                ColorMode::Always => self.convert_to_ansi(&line),
                _ => line.content(),
            })
            .collect::<Vec<String>>()
            .join("\n");
        (title, plaintext)
    }

    fn convert_to_ansi(&self, line: &Line) -> String {
        let mut painted = String::new();
        for span in &line.spans {
            if let Some(style) = span.style {
                let mut span_content = span.content.clone();
                if self.config.text_size_supported() {
                    span_content = resize_text(&span_content, style.size);
                }
                let span_content = &Self::apply_to_text(&span_content, style);
                painted.push_str(span_content);
            } else {
                painted.push_str(span.content.as_ref());
            }
        }

        if self.config.text_size_supported() {
            painted = add_additional_lines(painted, &line.spans);
        }
        painted
    }

    fn apply_to_text(content: &str, is_style: Style) -> String {
        is_style.to_ansi_style().paint(content).to_string()
    }

    fn sanitize(html: &str) -> String {
        html.replace('\t', "    ").replace(['\r', '\u{feff}'], "")
    }
}

fn add_additional_lines(content: String, spans: &[Span]) -> String {
    let mut max_height = 0;
    spans
        .iter()
        .filter_map(|span| span.style)
        .filter_map(|style| style.size)
        .for_each(|size| match size {
            Size::Double => max_height = max(max_height, 1),
            Size::Triple => max_height = max(max_height, 2),
            _ => {}
        });
    match max_height {
        1 => format!("{content}\n"),
        2 => format!("{content}\n\n"),
        _ => content,
    }
}

fn resize_text(content: &str, size: Option<Size>) -> String {
    if let Some(size) = size {
        return match size {
            Size::Normal => format!("\x1b]66;s=1;{content}\x07"),
            Size::Double => format!("\x1b]66;s=2;{content}\x07"),
            Size::Triple => format!("\x1b]66;s=3;{content}\x07"),
            Size::Half => {
                let mut result = String::new();
                let chars = content.chars().enumerate();

                for (i, c) in chars {
                    result.push(c);
                    if (i + 1) % 2 == 0 && (i + 1) != content.len() {
                        result.push_str("\x07\x1b]66;n=1:d=2:w=1;");
                    }
                }
                format!("\x1b]66;n=1:d=2:w=1;{result}\x07")
            }
        };
    }
    content.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search_engine::link::tests::TEST_CONFIG;
    use crate::search_engine::link::File;
    use crate::search_engine::link::HtmlSource::FileSource;
    use ctor::ctor;
    use globset::GlobSet;
    use std::collections::HashMap;
    use std::path::Path;

    impl PageExtractor {
        pub fn test_init(config: ExtractionConfig) -> Self {
            Self { config }
        }
    }

    #[ctor]
    fn setup() {
        TEST_CONFIG.write().format = Config::get_site_config("").format.clone();
        TEST_CONFIG.write().syntax = Config::get_site_config("").syntax.clone();
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
            true,
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
        assert_eq!(ansi_text.trim(), expected_content.trim());
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
            true,
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
            true,
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
            true,
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
            true,
        );
        let (_, text) = PageExtractor::test_init(config).get_tui_text(&source);

        let expected_lines: Vec<_> = expected_content.lines().collect();
        let result_lines: Vec<_> = text.iter().map(Line::content).collect();

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
            .iter()
            .flat_map(|line| line.spans.clone())
            .collect::<Vec<Span>>()
            .len();
        assert_eq!(length, 271);
    }
}
