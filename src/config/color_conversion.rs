use crate::errors::error::IsError;
use ratatui::prelude::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Deserialize, Clone, Default)]
pub struct TagStyleConfig {
    fg: Option<String>,
    bg: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    crossed_out: Option<bool>,
    dim: Option<bool>,
}

impl FromStr for TagStyleConfig {
    type Err = IsError;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut config = TagStyleConfig::default();

        string.split(';').for_each(|kv| {
            let kv = kv.trim();
            let mut split = kv.split('=');
            if let Some(key) = split.next() {
                let value = split.next().map(|s| s.trim().to_string());
                match key.trim().to_lowercase().as_str() {
                    "fg" => config.fg = value,
                    "bg" => config.bg = value,
                    "bold" => config.bold = Self::parse_bool(value),
                    "italic" => config.italic = Self::parse_bool(value),
                    "underlined" => config.underlined = Self::parse_bool(value),
                    "crossed_out" => config.crossed_out = Self::parse_bool(value),
                    "dim" => config.dim = Self::parse_bool(value),
                    _ => {
                        if !key.trim().is_empty() {
                            log::error!("Unrecognized tag style key: {}", key);
                        }
                    }
                }
            }
        });
        Ok(config)
    }
}

impl TagStyleConfig {
    fn parse_bool(value: Option<String>) -> Option<bool> {
        match value.as_deref() {
            None => Some(true),
            Some("true") => Some(true),
            Some(_) => Some(false),
        }
    }
}

pub fn convert_styles(styles: &HashMap<String, TagStyleConfig>) -> HashMap<String, Style> {
    styles
        .iter()
        .map(|(tag, cfg)| {
            let mut style = Style::default();
            if let Some(fg) = &cfg.fg {
                style = style.fg(parse_color(fg));
            }
            if let Some(bg) = &cfg.bg {
                style = style.bg(parse_color(bg));
            }
            let modifiers = [
                (cfg.bold, Modifier::BOLD),
                (cfg.italic, Modifier::ITALIC),
                (cfg.underlined, Modifier::UNDERLINED),
                (cfg.crossed_out, Modifier::CROSSED_OUT),
                (cfg.dim, Modifier::DIM),
            ];

            for (enabled, modifier) in modifiers {
                if enabled.unwrap_or(false) {
                    style = style.add_modifier(modifier);
                }
            }

            (tag.to_string(), style)
        })
        .collect()
}

pub fn parse_color(color: &str) -> Color {
    match color.to_lowercase().as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" => Color::Gray,
        "darkgray" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        _ => {
            if let Some(rgb) = parse_rgb(color) {
                rgb
            } else {
                Color::Reset
            }
        }
    }
}

fn parse_rgb(color: &str) -> Option<Color> {
    if let Some(hex) = color.strip_prefix('#') {
        if hex.len() == 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    } else if let Some(rgb_values) = color.strip_prefix("rgb(").and_then(|c| c.strip_suffix(")")) {
        let parts: Vec<&str> = rgb_values.split(',').map(str::trim).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }
    None
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rgb_valid_hex() {
        assert_eq!(parse_rgb("#ff5733"), Some(Color::Rgb(255, 87, 51)));
        assert_eq!(parse_rgb("#000000"), Some(Color::Rgb(0, 0, 0)));
        assert_eq!(parse_rgb("#FFFFFF"), Some(Color::Rgb(255, 255, 255)));
    }

    #[test]
    fn test_parse_rgb_invalid_hex() {
        assert_eq!(parse_rgb("#GGGGGG"), None);
        assert_eq!(parse_rgb("#12345"), None);
        assert_eq!(parse_rgb("123456"), None);
    }

    #[test]
    fn test_parse_rgb_valid_rgb_function() {
        assert_eq!(parse_rgb("rgb(255, 0, 0)"), Some(Color::Rgb(255, 0, 0)));
        assert_eq!(parse_rgb("rgb(0, 255, 128)"), Some(Color::Rgb(0, 255, 128)));
        assert_eq!(parse_rgb("rgb(12,34,56)"), Some(Color::Rgb(12, 34, 56)));
    }

    #[test]
    fn test_parse_rgb_invalid_rgb_function() {
        assert_eq!(parse_rgb("rgb(256, 0, 0)"), None);
        assert_eq!(parse_rgb("rgb(-1, 255, 0)"), None);
        assert_eq!(parse_rgb("rgb(0, 255)"), None);
        assert_eq!(parse_rgb("rgb(0, 255, abc)"), None);
    }

    #[test]
    fn test_convert_styles() {
        let mut styles = HashMap::new();
        styles.insert(
            "error".to_string(),
            TagStyleConfig {
                fg: Some("red".to_string()),
                bg: Some("#000000".to_string()),
                bold: Some(true),
                italic: Some(false),
                underlined: Some(true),
                crossed_out: None,
                dim: Some(false),
            },
        );

        let converted = convert_styles(&styles);
        let error_style = converted.get("error").unwrap();

        assert_eq!(error_style.fg, Some(Color::Red));
        assert_eq!(error_style.bg, Some(Color::Rgb(0, 0, 0)));

        error_style.add_modifier.contains(Modifier::UNDERLINED);

        assert!(error_style.add_modifier.contains(Modifier::BOLD));
        assert!(!error_style.add_modifier.contains(Modifier::ITALIC));
        assert!(error_style.add_modifier.contains(Modifier::UNDERLINED));
        assert!(!error_style.add_modifier.contains(Modifier::DIM));
    }

    #[test]
    fn test_valid_single_property() {
        let config = TagStyleConfig::from_str("fg=red").unwrap();
        assert_eq!(config.fg, Some("red".to_string()));
        assert_eq!(config.bg, None);
    }

    #[test]
    fn test_multiple_properties() {
        let config = TagStyleConfig::from_str("fg=blue; bg=black; bold=true").unwrap();
        assert_eq!(config.fg, Some("blue".to_string()));
        assert_eq!(config.bg, Some("black".to_string()));
        assert_eq!(config.bold, Some(true));
    }

    #[test]
    fn test_trailing_semicolon() {
        let config = TagStyleConfig::from_str("fg=green;").unwrap();
        assert_eq!(config.fg, Some("green".to_string()));
    }

    #[test]
    fn test_spaces_around_equals() {
        let config = TagStyleConfig::from_str(" fg = yellow ; bg = white ").unwrap();
        assert_eq!(config.fg, Some("yellow".to_string()));
        assert_eq!(config.bg, Some("white".to_string()));
    }

    #[test]
    fn test_invalid_property_ignored() {
        let config = TagStyleConfig::from_str("unknown=somevalue; fg=purple").unwrap();
        assert_eq!(config.fg, Some("purple".to_string()));
    }

    #[test]
    fn test_missing_value() {
        let config = TagStyleConfig::from_str("fg=; bg=white").unwrap();
        assert_eq!(config.fg, Some("".to_string())); // Empty value still valid
        assert_eq!(config.bg, Some("white".to_string()));
    }

    #[test]
    fn test_boolean_parsing() {
        let config = TagStyleConfig::from_str("bold=true; italic=false; underlined").unwrap();
        assert_eq!(config.bold, Some(true));
        assert_eq!(config.italic, Some(false));
        assert_eq!(config.underlined, Some(true)); // Missing value should default to true
    }
}
