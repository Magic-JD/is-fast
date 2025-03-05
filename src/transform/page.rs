use crate::errors::error::IsError;
use crate::errors::error::IsError::General;
use crate::transform::filter::filter;
use crate::transform::format::to_display;
use crate::transform::link::Link;
use crate::transform::scrape::scrape;
use ratatui::prelude::Text;
use ratatui::widgets::Paragraph;
use scraper::Html;
use std::fs;
use std::sync::Arc;

type ToHtml = Arc<dyn Fn(&Link) -> Result<String, IsError> + Send + Sync + 'static>;

#[derive(Clone)]
pub struct PageExtractor {
    pub convert_to_html: ToHtml,
}

impl PageExtractor {
    pub fn from_url() -> Self {
        Self {
            convert_to_html: Arc::new(|link| scrape(&link.url)),
        }
    }

    pub fn from_file() -> Self {
        Self {
            convert_to_html: Arc::new(|link| {
                fs::read_to_string(&link.title).map_err(|e| General(e.to_string()))
            }),
        }
    }

    pub fn get_paragraph(&self, link: &Link) -> Paragraph<'static> {
        Paragraph::new(self.get_tui_text(link))
    }
    fn get_tui_text(&self, link: &Link) -> Text<'static> {
        (self.convert_to_html)(link)
            .map(|html| PageExtractor::sanitize(&html))
            .map(|res| Html::parse_document(res.as_str()))
            .and_then(|html| filter(&html, &link.selector).and_then(to_display))
            .unwrap_or_else(|_| Text::from("Failed to convert to text"))
    }

    pub fn get_plain_text(&self, link: &Link) -> String {
        self.get_tui_text(link)
            .lines
            .iter()
            .map(|line| line.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    }

    fn sanitize(html: &str) -> String {
        html.replace("\t", "    ")
            .replace("\r", "")
            .replace('\u{feff}', "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::text::Span;
    use std::path::Path;

    #[test]
    fn test_large_scale_file_as_expected() {
        let path_sample = String::from("tests/data/sample.html");
        let link = Link::new(path_sample, String::default(), String::from("body"));
        let plain_text = PageExtractor::from_file()
            .get_plain_text(&link)
            .trim()
            .to_owned();

        let path_output = Path::new("tests/data/expected_text.txt");
        let expected_content = fs::read_to_string(path_output)
            .expect("Failed to read test HTML file")
            .to_owned();

        assert_eq!(plain_text, expected_content);

        let result = PageExtractor::from_file().get_tui_text(&link);

        let expected_lines: Vec<_> = expected_content.lines().collect();
        let binding = result.to_string();
        let result_lines: Vec<_> = binding.lines().collect();

        let min_len = expected_lines.len().min(result_lines.len());

        for i in 0..min_len {
            if expected_lines[i] != result_lines[i + 1] {
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
        assert_eq!(length, 556);
    }
}
