use crate::config::color_conversion::Style;

use ratatui::text::{Line as RatLine, Span as RatSpan};

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Line {
    pub spans: Vec<Span>,
    pub style: Option<Style>,
}

impl Line {
    pub fn to_rat_colorless(&self) -> RatLine<'static> {
        let text = self
            .spans
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<&str>>()
            .join("")
            .to_string();
        RatLine::from(text)
    }

    pub fn to_rat_colored(&self) -> RatLine<'static> {
        let content = self
            .spans
            .iter()
            .map(Span::to_rat_span)
            .collect::<Vec<RatSpan>>();
        let mut rat_line = RatLine::from(content);
        if let Some(style) = &self.style {
            rat_line.style = style.to_rat_style();
        }
        rat_line
    }

    pub fn from(spans: Vec<Span>) -> Self {
        Self { spans, style: None }
    }

    pub fn from_single(span: Span) -> Self {
        Self {
            spans: vec![span],
            style: None,
        }
    }

    pub fn set_style(self, style: Style) -> Self {
        let mut new_line = self.clone();
        new_line.style = Some(style);
        new_line
    }

    pub(crate) fn content(&self) -> String {
        self.spans
            .iter()
            .map(|s| s.content.as_str())
            .collect::<Vec<&str>>()
            .join("")
            .to_string()
    }
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Span {
    pub content: String,
    pub style: Option<Style>,
}

impl Span {
    pub fn from(content: &str) -> Self {
        Self {
            content: content.replace('\n', "").to_string(),
            style: None,
        }
    }

    pub fn styled(content: &str, style: Style) -> Self {
        Self {
            content: content.replace('\n', "").to_string(),
            style: Some(style),
        }
    }

    pub fn to_rat_span(&self) -> RatSpan<'static> {
        let style = self.style.as_ref().map(|s| s.to_rat_style());
        if let Some(style) = style {
            RatSpan::styled(self.content.clone(), style)
        } else {
            RatSpan::raw(self.content.clone())
        }
    }
}
