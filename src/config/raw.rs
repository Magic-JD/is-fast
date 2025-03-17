use globset::{Glob, GlobSet, GlobSetBuilder};
use ratatui::prelude::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;
use std::{env, fs};

#[derive(Debug, Deserialize)]
pub struct TagStyleConfig {
    fg: Option<String>,
    bg: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    underlined: Option<bool>,
    crossed_out: Option<bool>,
    dim: Option<bool>,
}
#[derive(Debug, Deserialize)]
pub struct FormatSection {
    #[serde(default)]
    pub(crate) ignored_tags: Vec<String>,
    #[serde(default)]
    pub(crate) block_elements: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SyntaxHighlightingSection {
    #[serde(default)]
    pub(crate) theme: Option<String>,
    #[serde(default)]
    pub(crate) default_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DisplaySection {
    #[serde(default)]
    pub(crate) border_color: Option<String>,
    #[serde(default)]
    pub(crate) page_margin: Option<u16>,
    #[serde(default)]
    pub(crate) scroll: Option<String>,
    #[serde(default)]
    pub(crate) color_mode: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct HistorySection {
    #[serde(default)]
    pub(crate) title_color: Option<String>,
    #[serde(default)]
    pub(crate) url_color: Option<String>,
    #[serde(default)]
    pub(crate) time_color: Option<String>,
    #[serde(default)]
    pub(crate) text_color: Option<String>,
    #[serde(default)]
    pub(crate) search_type: Option<String>,
    #[serde(default)]
    pub(crate) enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct SearchSection {
    #[serde(default)]
    pub(crate) engine: Option<String>,
    #[serde(default)]
    pub(crate) site: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CacheSection {
    #[serde(default)]
    pub(crate) cache_mode: Option<String>,
    #[serde(default)]
    pub(crate) max_size: Option<usize>,
    #[serde(default)]
    pub(crate) ttl: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiscSection {
    #[serde(default)]
    pub(crate) open_tool: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RawConfig {
    #[serde(default)]
    pub(crate) styles: HashMap<String, TagStyleConfig>,
    #[serde(default)]
    pub(crate) selectors: HashMap<String, String>,
    #[serde(default)]
    pub(crate) format: Option<FormatSection>,
    #[serde(default)]
    pub(crate) syntax: Option<SyntaxHighlightingSection>,
    #[serde(default)]
    pub(crate) display: Option<DisplaySection>,
    #[serde(default)]
    pub(crate) history: Option<HistorySection>,
    #[serde(default)]
    pub(crate) search: Option<SearchSection>,
    #[serde(default)]
    pub(crate) cache: Option<CacheSection>,
    #[serde(default)]
    pub(crate) misc: Option<MiscSection>,
}

impl RawConfig {
    pub fn default() -> Self {
        Self {
            styles: HashMap::new(),
            selectors: HashMap::new(),
            format: None,
            syntax: None,
            display: None,
            history: None,
            search: None,
            cache: None,
            misc: None,
        }
    }
}

pub fn override_defaults(config: &mut RawConfig, u_config: RawConfig) {
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

    let mut display = config.display.take().unwrap_or(DisplaySection {
        border_color: None,
        page_margin: None,
        scroll: None,
        color_mode: None,
    });
    if let Some(u_display) = u_config.display {
        if let Some(border_color) = u_display.border_color {
            display.border_color = Some(border_color);
        }
        if let Some(margin) = u_display.page_margin {
            if margin < 50 {
                display.page_margin = Some(margin);
            }
        }
        if let Some(scroll) = u_display.scroll {
            display.scroll = Some(scroll);
        }
        if let Some(color_mode) = u_display.color_mode {
            display.color_mode = Some(color_mode);
        }
    }

    let mut history = config.history.take().unwrap_or(HistorySection {
        title_color: None,
        url_color: None,
        time_color: None,
        text_color: None,
        search_type: None,
        enabled: None,
    });

    if let Some(u_history) = u_config.history {
        if let Some(title_color) = u_history.title_color {
            history.title_color = Some(title_color);
        }
        if let Some(url_color) = u_history.url_color {
            history.url_color = Some(url_color);
        }
        if let Some(time_color) = u_history.time_color {
            history.time_color = Some(time_color);
        }
        if let Some(text_color) = u_history.text_color {
            history.text_color = Some(text_color);
        }
        if let Some(search_type) = u_history.search_type {
            history.search_type = Some(search_type);
        }
        if let Some(enabled) = u_history.enabled {
            history.enabled = Some(enabled);
        }
    }

    let mut search = config.search.take().unwrap_or(SearchSection {
        engine: None,
        site: None,
    });

    if let Some(u_search) = u_config.search {
        if let Some(engine) = u_search.engine {
            search.engine = Some(engine);
        }
        if let Some(site) = u_search.site {
            search.site = Some(site);
        }
    }

    let mut cache = config.cache.take().unwrap_or(CacheSection {
        cache_mode: None,
        max_size: None,
        ttl: None,
    });
    if let Some(u_cache) = u_config.cache {
        if let Some(cache_mode) = u_cache.cache_mode {
            cache.cache_mode = Some(cache_mode);
        }
        if let Some(max_size) = u_cache.max_size {
            cache.max_size = Some(max_size);
        }
        if let Some(ttl) = u_cache.ttl {
            cache.ttl = Some(ttl);
        }
    }

    let mut misc = config
        .misc
        .take()
        .unwrap_or(MiscSection { open_tool: None });

    if let Some(u_misc) = u_config.misc {
        if let Some(open_tool) = u_misc.open_tool {
            misc.open_tool = Some(open_tool);
        }
    }

    config.search = Some(search);
    config.history = Some(history);
    config.format = Some(format);
    config.syntax = Some(syntax);
    config.display = Some(display);
    config.cache = Some(cache);
    config.misc = Some(misc);
}

pub fn get_user_specified_config() -> Option<RawConfig> {
    env::var("IS_FAST_CONFIG_PATH")
        .ok()
        .map(PathBuf::from)
        .and_then(config_from_filepath)
        .or_else(|| {
            dirs::config_dir()
                .map(|p| p.join("is-fast/config.toml"))
                .map_or_else(|| None, config_from_filepath)
        })
}

fn config_from_filepath(buff: PathBuf) -> Option<RawConfig> {
    fs::read_to_string(buff)
        .ok()
        .as_ref()
        .and_then(|str| toml::from_str(str).ok())
}

pub fn generate_globs(config: &mut RawConfig) -> (GlobSet, Vec<Glob>) {
    let mut builder = GlobSetBuilder::new();
    let mut globs = Vec::new();
    config.selectors.iter().for_each(|(pattern, _)| {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob.clone());
            globs.push(glob);
        }
    });
    let matcher = builder
        .build()
        .inspect_err(|err| log::error!("{err} : cannot build glob matcher."))
        .unwrap_or_default(); // Should be safe as only valid globs added
    (matcher, globs)
}
pub fn convert_styles(styles: HashMap<String, TagStyleConfig>) -> HashMap<String, Style> {
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
    use crate::config::raw::RawConfig;

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
            display: None,
            history: None,
            search: None,
            cache: None,
            misc: None,
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
            display: Some(DisplaySection {
                border_color: Some("green".to_string()),
                page_margin: Some(3),
                scroll: None,
                color_mode: None,
            }),
            history: Some(HistorySection {
                title_color: Some("blue".to_string()),
                url_color: Some("cyan".to_string()),
                time_color: Some("gray".to_string()),
                text_color: Some("white".to_string()),
                search_type: Some("fuzzy".to_string()),
                enabled: None,
            }),
            search: None,
            cache: None,
            misc: None,
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
            display: Some(DisplaySection {
                border_color: Some("yellow".to_string()),
                page_margin: Some(5),
                scroll: None,
                color_mode: None,
            }),
            history: Some(HistorySection {
                title_color: Some("red".to_string()),
                url_color: None,
                time_color: Some("black".to_string()),
                text_color: None,
                search_type: Some("fuzzy".to_string()),
                enabled: None,
            }),
            search: None,
            cache: None,
            misc: None,
        };

        override_defaults(&mut default_config, user_config);

        // Selector Tests
        assert_eq!(default_config.selectors.len(), 2);
        assert!(default_config.selectors.contains_key("example.com"));
        assert!(default_config.selectors.contains_key("newsite.com"));

        // Format Tests
        assert_eq!(
            default_config.format.as_ref().unwrap().ignored_tags,
            vec!["style"]
        );
        assert_eq!(
            default_config.format.as_ref().unwrap().block_elements,
            vec!["div"]
        );

        // Syntax Highlighting Tests
        assert_eq!(
            default_config.syntax.as_ref().unwrap().theme,
            Some("light".to_string())
        );
        assert_eq!(
            default_config.syntax.as_ref().unwrap().default_language,
            Some("rust".to_string())
        );

        // Display Tests
        assert_eq!(
            default_config.display.as_ref().unwrap().border_color,
            Some("yellow".to_string())
        );
        assert_eq!(
            default_config.display.as_ref().unwrap().page_margin,
            Some(5)
        );

        // History Tests
        assert_eq!(
            default_config.history.as_ref().unwrap().title_color,
            Some("red".to_string())
        );
        assert_eq!(
            default_config.history.as_ref().unwrap().url_color,
            Some("cyan".to_string())
        );
        assert_eq!(
            default_config.history.as_ref().unwrap().time_color,
            Some("black".to_string())
        );
        assert_eq!(
            default_config.history.as_ref().unwrap().text_color,
            Some("white".to_string())
        );
        assert_eq!(
            default_config.history.as_ref().unwrap().search_type,
            Some("fuzzy".to_string())
        );
    }
}
