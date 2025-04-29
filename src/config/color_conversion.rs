use crate::errors::error::IsError;
use nu_ansi_term::{Color as AnsiColor, Style as AnsiStyle};
use ratatui::style::{Color as RatColor, Modifier, Style as RatStyle};
use serde::Deserialize;
use std::str::FromStr;
use syntect::highlighting::Color as SyntectColor;

#[derive(Debug, Deserialize, Copy, Clone, Default, PartialEq, Eq, Hash)]
#[serde(try_from = "String")]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl TryFrom<String> for Color {
    type Error = IsError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::parse_color(&s).ok_or(IsError::TagStyleError(format!("Invalid color: {s}")))
    }
}

impl FromStr for Color {
    type Err = IsError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse_color(s).ok_or(IsError::TagStyleError(format!("Invalid color: {s}")))
    }
}

impl Color {
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    fn parse_rgb(color: &str) -> Option<Color> {
        if let Some(hex) = color.strip_prefix('#') {
            if hex.len() == 6 {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                return Some(Color::rgb(r, g, b));
            }
        } else if let Some(rgb_values) =
            color.strip_prefix("rgb(").and_then(|c| c.strip_suffix(")"))
        {
            let parts: Vec<&str> = rgb_values.split(',').map(str::trim).collect();
            if parts.len() == 3 {
                let r = parts[0].parse::<u8>().ok()?;
                let g = parts[1].parse::<u8>().ok()?;
                let b = parts[2].parse::<u8>().ok()?;
                return Some(Color::rgb(r, g, b));
            }
        }
        None
    }

    fn parse_color(color: &str) -> Option<Color> {
        match color.to_lowercase().as_str() {
            "black" => Some(Color::rgb(0, 0, 0)),
            "red" => Some(Color::rgb(208, 84, 84)),
            "green" => Some(Color::rgb(88, 204, 84)),
            "yellow" => Some(Color::rgb(208, 204, 84)),
            "blue" => Some(Color::rgb(88, 84, 204)),
            "magenta" => Some(Color::rgb(208, 84, 204)),
            "cyan" => Some(Color::rgb(128, 204, 204)),
            "white" => Some(Color::rgb(255, 255, 255)),
            "gray" => Some(Color::rgb(208, 204, 204)),
            "darkgray" => Some(Color::rgb(88, 84, 84)),
            "lightred" => Some(Color::rgb(255, 84, 80)),
            "lightgreen" => Some(Color::rgb(88, 252, 84)),
            "lightyellow" => Some(Color::rgb(255, 255, 85)),
            "lightblue" => Some(Color::rgb(88, 84, 252)),
            "lightmagenta" => Some(Color::rgb(255, 84, 252)),
            "lightcyan" => Some(Color::rgb(88, 252, 252)),
            _ => Self::parse_rgb(color),
        }
    }

    pub fn to_rat_color(self) -> RatColor {
        RatColor::Rgb(self.r, self.g, self.b)
    }

    pub fn from_syntect_color(syntext_color: SyntectColor) -> Self {
        Self {
            r: syntext_color.r,
            g: syntext_color.g,
            b: syntext_color.b,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Size {
    #[serde(alias = "1", alias = "normal")]
    Normal,
    #[serde(alias = "2", alias = "double")]
    Double,
    #[serde(alias = "3", alias = "triple")]
    Triple,
    #[serde(alias = "half")]
    Half,
}

#[derive(Debug, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    pub(crate) size: Option<Size>,
    pub(crate) bold: Option<bool>,
    pub(crate) italic: Option<bool>,
    underlined: Option<bool>,
    crossed_out: Option<bool>,
    dim: Option<bool>,
}

impl Style {
    pub(crate) fn parse_size(value: Option<&str>) -> Option<Size> {
        match value {
            Some("half") => Some(Size::Half),
            Some("1" | "normal") => Some(Size::Normal),
            Some("2" | "double") => Some(Size::Double),
            Some("3" | "triple") => Some(Size::Triple),
            _ => None,
        }
    }
}

impl FromStr for Style {
    type Err = IsError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut config = Style::default();

        string.split(';').for_each(|kv| {
            let kv = kv.trim();
            let mut split = kv.split('=');
            if let Some(key) = split.next() {
                let value = split.next().map(|s| s.trim().to_string());
                match key.trim().to_lowercase().as_str() {
                    "fg" => config.fg = value.and_then(|col| Color::from_str(col.as_str()).ok()),
                    "bg" => config.bg = value.and_then(|col| Color::from_str(col.as_str()).ok()),
                    "size" => config.size = Self::parse_size(value.as_deref()),
                    "bold" => config.bold = Self::parse_bool(value.as_deref()),
                    "italic" => config.italic = Self::parse_bool(value.as_deref()),
                    "underlined" => config.underlined = Self::parse_bool(value.as_deref()),
                    "crossed_out" => config.crossed_out = Self::parse_bool(value.as_deref()),
                    "dim" => config.dim = Self::parse_bool(value.as_deref()),
                    _ => {
                        if !key.trim().is_empty() {
                            log::error!("Unrecognized tag style key: {key}");
                        }
                    }
                }
            }
        });
        Ok(config)
    }
}

impl Style {
    fn parse_bool(value: Option<&str>) -> Option<bool> {
        match value {
            Some("true") | None => Some(true),
            Some("false") => Some(false),
            _ => None,
        }
    }

    pub fn to_rat_style(self) -> RatStyle {
        let mut style = RatStyle::default();
        if let Some(fg) = &self.fg {
            style = style.fg(fg.to_rat_color());
        }
        if let Some(bg) = &self.bg {
            style = style.bg(bg.to_rat_color());
        }
        let modifiers = [
            (self.bold, Modifier::BOLD),
            (self.italic, Modifier::ITALIC),
            (self.underlined, Modifier::UNDERLINED),
            (self.crossed_out, Modifier::CROSSED_OUT),
            (self.dim, Modifier::DIM),
        ];

        for (enabled, modifier) in modifiers {
            if enabled.unwrap_or(false) {
                style = style.add_modifier(modifier);
            }
        }
        style
    }

    pub fn fg(color: Color) -> Self {
        Self {
            fg: Some(color),
            bg: None,
            size: None,
            bold: None,
            italic: None,
            underlined: None,
            crossed_out: None,
            dim: None,
        }
    }

    pub fn patch(&self, other: &Self) -> Self {
        Self {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            size: other.size.or(self.size),
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            underlined: other.underlined.or(self.underlined),
            crossed_out: other.crossed_out.or(self.crossed_out),
            dim: other.dim.or(self.dim),
        }
    }

    pub fn to_ansi_style(self) -> AnsiStyle {
        let mut style = AnsiStyle::new();
        if let Some(is_color) = &self.fg {
            let color = AnsiColor::Rgb(is_color.r, is_color.g, is_color.b);
            style = style.fg(color);
        }
        if let Some(is_color) = &self.bg {
            let color = AnsiColor::Rgb(is_color.r, is_color.g, is_color.b);
            style = style.on(color);
        }
        self.bold.inspect(|b| {
            if *b {
                style = style.bold();
            }
        });
        self.italic.inspect(|b| {
            if *b {
                style = style.italic();
            }
        });
        self.underlined.inspect(|b| {
            if *b {
                style = style.underline();
            }
        });
        self.dim.inspect(|b| {
            if *b {
                style = style.dimmed();
            }
        });
        self.crossed_out.inspect(|b| {
            if *b {
                style = style.strikethrough();
            }
        });
        style
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rgb_valid_hex() {
        assert_eq!(
            Color::from_str("#ff5733").ok(),
            Some(Color::rgb(255, 87, 51))
        );
        assert_eq!(Color::from_str("#000000").ok(), Some(Color::rgb(0, 0, 0)));
        assert_eq!(
            Color::from_str("#FFFFFF").ok(),
            Some(Color::rgb(255, 255, 255))
        );
    }

    #[test]
    fn test_parse_rgb_invalid_hex() {
        assert_eq!(Color::from_str("#GGGGGG").ok(), None);
        assert_eq!(Color::from_str("#12345").ok(), None);
        assert_eq!(Color::from_str("123456").ok(), None);
    }

    #[test]
    fn test_parse_rgb_valid_rgb_function() {
        assert_eq!(
            Color::from_str("rgb(255, 0, 0)").ok(),
            Some(Color::rgb(255, 0, 0))
        );
        assert_eq!(
            Color::from_str("rgb(0, 255, 128)").ok(),
            Some(Color::rgb(0, 255, 128))
        );
        assert_eq!(
            Color::from_str("rgb(12,34,56)").ok(),
            Some(Color::rgb(12, 34, 56))
        );
    }

    #[test]
    fn test_parse_rgb_invalid_rgb_function() {
        assert_eq!(Color::from_str("rgb(256, 0, 0)").ok(), None);
        assert_eq!(Color::from_str("rgb(-1, 255, 0)").ok(), None);
        assert_eq!(Color::from_str("rgb(0, 255)").ok(), None);
        assert_eq!(Color::from_str("rgb(0, 255, abc)").ok(), None);
    }

    #[test]
    fn test_convert_styles_rat() {
        let style = Style {
            fg: Color::from_str("red").ok(),
            bg: Color::from_str("#000000").ok(),
            size: None,
            bold: Some(true),
            italic: Some(false),
            underlined: Some(true),
            crossed_out: None,
            dim: Some(false),
        };

        let error_style = style.to_rat_style();

        assert_eq!(error_style.fg, Some(RatColor::Rgb(208, 84, 84)));
        assert_eq!(error_style.bg, Some(RatColor::Rgb(0, 0, 0)));

        assert!(error_style.add_modifier.contains(Modifier::BOLD));
        assert!(!error_style.add_modifier.contains(Modifier::ITALIC));
        assert!(error_style.add_modifier.contains(Modifier::UNDERLINED));
        assert!(!error_style.add_modifier.contains(Modifier::CROSSED_OUT));
        assert!(!error_style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn test_convert_styles_ansi() {
        let style = Style {
            fg: Color::from_str("red").ok(),
            bg: Color::from_str("#000000").ok(),
            size: None,
            bold: Some(true),
            italic: Some(true),
            underlined: Some(true),
            crossed_out: Some(true),
            dim: Some(true),
        };

        let error_style = style.to_ansi_style();

        assert_eq!(error_style.foreground, Some(AnsiColor::Rgb(208, 84, 84)));
        assert_eq!(error_style.background, Some(AnsiColor::Rgb(0, 0, 0)));

        assert!(error_style.is_underline);
        assert!(error_style.is_bold);
        assert!(error_style.is_italic);
        assert!(error_style.is_strikethrough);
        assert!(error_style.is_dimmed);
    }

    #[test]
    fn test_convert_styles_ansi_default() {
        let style = Style::default();
        let error_style = style.to_ansi_style();

        assert_eq!(error_style.foreground, None);
        assert_eq!(error_style.background, None);

        assert!(!error_style.is_underline);
        assert!(!error_style.is_bold);
        assert!(!error_style.is_italic);
        assert!(!error_style.is_strikethrough);
        assert!(!error_style.is_dimmed);
    }

    #[test]
    fn test_valid_single_property() {
        let config = Style::from_str("fg=red").unwrap();
        assert_eq!(config.fg, Color::from_str("red").ok());
        assert_eq!(config.bg, None);
    }

    #[test]
    fn test_multiple_properties() {
        let config = Style::from_str("fg=blue; bg=black; bold=true").unwrap();
        assert_eq!(config.fg, Color::from_str("blue").ok());
        assert_eq!(config.bg, Color::from_str("black").ok());
        assert_eq!(config.bold, Some(true));
    }

    #[test]
    fn test_trailing_semicolon() {
        let config = Style::from_str("fg=green;").unwrap();
        assert_eq!(config.fg, Color::from_str("green").ok());
    }

    #[test]
    fn test_spaces_around_equals() {
        let config = Style::from_str(" fg = yellow ; bg = white ").unwrap();
        assert_eq!(config.fg, Color::from_str("yellow").ok());
        assert_eq!(config.bg, Color::from_str("white").ok());
    }

    #[test]
    fn test_invalid_property_ignored() {
        let config = Style::from_str("unknown=somevalue; fg=purple").unwrap();
        assert_eq!(config.fg, Color::from_str("purple").ok());
    }

    #[test]
    fn test_missing_value() {
        let config = Style::from_str("fg=; bg=white").unwrap();
        assert_eq!(config.fg, None);
        assert_eq!(config.bg, Color::from_str("white").ok());
    }

    #[test]
    fn test_boolean_parsing() {
        let config = Style::from_str("bold=true; italic=false; underlined").unwrap();
        assert_eq!(config.bold, Some(true));
        assert_eq!(config.italic, Some(false));
        assert_eq!(config.underlined, Some(true));
    }
}
