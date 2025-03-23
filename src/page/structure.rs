use crate::config::color_conversion::Style;

use ratatui::text::{Line as RatLine, Span as RatSpan};

#[derive(Default, PartialEq, Debug, Clone)]
pub struct Line {
    pub spans: Vec<Span>,
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
        RatLine::from(content)
    }

    pub fn from(spans: Vec<Span>) -> Self {
        Self { spans }
    }

    pub fn from_single(span: Span) -> Self {
        Self { spans: vec![span] }
    }

    pub fn set_style(self, style: Style) -> Self {
        Line::from(
            self.spans
                .into_iter()
                .map(|mut span| {
                    span.style = span
                        .style
                        .map(|old_style: Style| style.patch(&old_style))
                        .or(Some(style));
                    span
                })
                .collect(),
        )
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
