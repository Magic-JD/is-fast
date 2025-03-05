use globset::{Glob, GlobSet, GlobSetBuilder};
use once_cell::sync::Lazy;
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::{env, fs};
use toml;

static CONFIG: Lazy<Config> = Lazy::new(Config::load);
pub const DEFAULT_CONFIG_LOCATION: &str = include_str!("config.toml");

#[derive(Debug, Deserialize)]
struct TagStyleConfig {
    fg: Option<String>,
    bg: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    crossed_out: Option<bool>,
    dim: Option<bool>,
}
#[derive(Debug, Deserialize)]
struct FormatSection {
    #[serde(default)]
    ignored_tags: Vec<String>,
    #[serde(default)]
    block_elements: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct SyntaxHighlightingSection {
    #[serde(default)]
    theme: Option<String>,
    #[serde(default)]
    default_language: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawConfig {
    #[serde(default)]
    styles: HashMap<String, TagStyleConfig>,
    #[serde(default)]
    selectors: HashMap<String, String>,
    #[serde(default)]
    format: Option<FormatSection>,
    #[serde(default)]
    syntax: Option<SyntaxHighlightingSection>,
}

#[derive(Debug)]
pub struct Config {
    styles: HashMap<String, Style>,
    selectors: HashMap<String, String>,
    matcher: GlobSet,
    globs: Vec<Glob>,
    ignored_tags: HashSet<String>,
    block_elements: HashSet<String>,
    syntax_default_language: Option<String>,
    syntax_highlighting_theme: Option<String>,
}

impl Config {
    fn load() -> Self {
        let mut config: RawConfig = toml::from_str(DEFAULT_CONFIG_LOCATION)
            .map_err(|e| println!("{}", e))
            .unwrap_or(RawConfig {
                styles: HashMap::new(),
                selectors: HashMap::new(),
                format: None,
                syntax: None,
            });
        _ = get_user_specified_config().map(|u_config| override_defaults(&mut config, u_config));
        let (matcher, globs) = generate_globs(&mut config);
        Self {
            styles: convert_styles(config.styles),
            selectors: config.selectors,
            globs,
            matcher,
            ignored_tags: config
                .format
                .as_ref()
                .map(|format| format.ignored_tags.iter().cloned().collect())
                .unwrap_or_default(),
            block_elements: config
                .format
                .as_ref()
                .map(|format| format.block_elements.iter().cloned().collect())
                .unwrap_or_default(),
            syntax_default_language: config
                .syntax
                .as_ref()
                .and_then(|syntax| syntax.default_language.clone()),
            syntax_highlighting_theme: config
                .syntax
                .as_ref()
                .and_then(|syntax| syntax.theme.clone()),
        }
    }

    pub fn get_styles() -> HashMap<String, Style> {
        CONFIG.styles.clone()
    }

    pub fn get_selectors(url: &str) -> String {
        CONFIG
            .matcher
            .matches(url)
            .iter()
            .find_map(|idx| CONFIG.globs.get(*idx))
            .map(|glob| glob.clone().to_string())
            .and_then(|s| CONFIG.selectors.get(&s).cloned())
            .unwrap_or_else(|| String::from("body"))
    }

    pub fn get_ignored_tags() -> HashSet<String> {
        CONFIG.ignored_tags.clone()
    }

    pub fn get_block_elements() -> HashSet<String> {
        CONFIG.block_elements.clone()
    }

    pub fn get_default_language() -> String {
        CONFIG.syntax_default_language.clone().unwrap_or_default()
    }

    pub fn get_syntax_highlighting_theme() -> String {
        CONFIG
            .syntax_highlighting_theme
            .clone()
            .unwrap_or_else(|| "base16-ocean.dark".to_string())
    }
}

fn generate_globs(config: &mut RawConfig) -> (GlobSet, Vec<Glob>) {
    let mut builder = GlobSetBuilder::new();
    let mut globs = Vec::new();
    config.selectors.iter().for_each(|(pattern, _)| {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob.clone());
            globs.push(glob);
        }
    });
    let matcher = builder.build().unwrap(); // Should be safe as only valid globs added
    (matcher, globs)
}

fn override_defaults(config: &mut RawConfig, u_config: RawConfig) {
    for (tag, user_style) in u_config.styles {
        config.styles.insert(tag, user_style);
    }
    for (site, selector) in u_config.selectors {
        config.selectors.insert(site, selector);
    }
    let mut format = config.format.take().unwrap_or_else(|| FormatSection {
        ignored_tags: Vec::new(),
        block_elements: Vec::new(),
    });

    if let Some(u_format) = u_config.format {
        if !u_format.ignored_tags.is_empty() {
            format.ignored_tags = u_format.ignored_tags;
        }
        if !u_format.block_elements.is_empty() {
            format.block_elements = u_format.block_elements;
        }
    }
    let mut syntax = config.syntax.take().unwrap_or(SyntaxHighlightingSection {
        theme: None,
        default_language: None,
    });
    if let Some(u_syntax) = u_config.syntax {
        if let Some(theme) = u_syntax.theme {
            syntax.theme = Some(theme);
        }
        if let Some(default_language) = u_syntax.default_language {
            syntax.default_language = Some(default_language);
        }
    }
    config.format = Some(format);
    config.syntax = Some(syntax);
}

fn parse_color(color: &str) -> Color {
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
        let parts: Vec<&str> = rgb_values.split(',').map(|s| s.trim()).collect();
        if parts.len() == 3 {
            let r = parts[0].parse::<u8>().ok()?;
            let g = parts[1].parse::<u8>().ok()?;
            let b = parts[2].parse::<u8>().ok()?;
            return Some(Color::Rgb(r, g, b));
        }
    }
    None
}

fn get_user_specified_config() -> Option<RawConfig> {
    env::var("IS_FAST_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .and_then(config_from_filepath)
        .or_else(|| {
            dirs::config_dir()
                .map(|p| p.join("is-fast/config.toml"))
                .map(config_from_filepath)
                .unwrap_or_else(|| None)
        })
}

fn config_from_filepath(buff: PathBuf) -> Option<RawConfig> {
    fs::read_to_string(buff)
        .ok()
        .as_ref()
        .and_then(|str| toml::from_str(str).ok())
}

fn convert_styles(styles: HashMap<String, TagStyleConfig>) -> HashMap<String, Style> {
    styles
        .into_iter()
        .map(|(tag, cfg)| {
            let mut style = Style::default();
            if let Some(fg) = cfg.fg {
                style = style.fg(parse_color(&fg));
            }
            if let Some(bg) = cfg.bg {
                style = style.bg(parse_color(&bg));
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

            (tag, style)
        })
        .collect()
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

        let converted = convert_styles(styles);
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
    fn test_generate_globs() {
        let mut raw_config = RawConfig {
            selectors: {
                let mut map = HashMap::new();
                map.insert("example.com/*".to_string(), "body".to_string());
                map.insert("*.org".to_string(), "div".to_string());
                map
            },
            styles: HashMap::new(),
            format: None,
            syntax: None,
        };

        let (matcher, globs) = generate_globs(&mut raw_config);

        assert_eq!(globs.len(), 2);
        assert!(matcher.is_match("example.com/index.html"));
        assert!(matcher.is_match("test.org"));
        assert!(!matcher.is_match("random.net"));
    }

    #[test]
    fn test_override_defaults() {
        let mut default_config = RawConfig {
            selectors: {
                let mut map = HashMap::new();
                map.insert("example.com".to_string(), "body".to_string());
                map
            },
            styles: HashMap::new(),
            format: Some(FormatSection {
                ignored_tags: vec!["script".to_string()],
                block_elements: vec!["div".to_string()],
            }),
            syntax: Some(SyntaxHighlightingSection {
                theme: Some("dark".to_string()),
                default_language: Some("rust".to_string()),
            }),
        };

        let user_config = RawConfig {
            selectors: {
                let mut map = HashMap::new();
                map.insert("newsite.com".to_string(), "header".to_string());
                map
            },
            styles: HashMap::new(),
            format: Some(FormatSection {
                ignored_tags: vec!["style".to_string()],
                block_elements: vec![],
            }),
            syntax: Some(SyntaxHighlightingSection {
                theme: Some("light".to_string()),
                default_language: None,
            }),
        };

        override_defaults(&mut default_config, user_config);

        assert_eq!(default_config.selectors.len(), 2);
        assert!(default_config.selectors.contains_key("example.com"));
        assert!(default_config.selectors.contains_key("newsite.com"));

        assert_eq!(
            default_config.format.as_ref().unwrap().ignored_tags,
            vec!["style"]
        );
        assert_eq!(
            default_config.format.as_ref().unwrap().block_elements,
            vec!["div"]
        );

        assert_eq!(
            default_config.syntax.as_ref().unwrap().theme,
            Some("light".to_string())
        );
        assert_eq!(
            default_config.syntax.as_ref().unwrap().default_language,
            Some("rust".to_string())
        );
    }
}
