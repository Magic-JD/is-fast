use once_cell::sync::Lazy;
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use toml;

static CONFIG: Lazy<Config> = Lazy::new(Config::load);
const DEFAULT_CONFIG: &str = include_str!("config/config.toml");

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
struct RawConfig {
    #[serde(default)]
    styles: HashMap<String, TagStyleConfig>,
    #[serde(default)]
    selectors: HashMap<String, String>,
    #[serde(default)]
    format: Option<FormatSection>,
}

#[derive(Debug)]
pub struct Config {
    styles: HashMap<String, Style>,
    selectors: HashMap<String, String>,
    ignored_tags: HashSet<String>,
    block_elements: HashSet<String>,
}

impl Config {
    fn load() -> Self {
        let mut config: RawConfig = toml::from_str(DEFAULT_CONFIG).map_err(|e|
                                                                               {
                                                                                   println!("{}", e.to_string())
                                                                               }
                                                                                   ).unwrap_or(RawConfig {
            styles: HashMap::new(),
            selectors: HashMap::new(),
            format: None,
        });
        _ = dirs::config_dir()
            .map(|p| p.join("is-fast/config.toml"))
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| toml::from_str::<RawConfig>(&content).ok())
            .map(|u_config| {
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
                config.format = Some(format);
            });

        Self {
            styles: Self::convert_styles(config.styles),
            selectors: config.selectors,
            ignored_tags: config.format.as_ref().map(|format| format.ignored_tags.iter().cloned().collect()).unwrap_or_else(|| HashSet::new()),
            block_elements: config.format.as_ref().map(|format| format.block_elements.iter().cloned().collect()).unwrap_or_else(|| HashSet::new()),
        }
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
                if cfg.bold.unwrap_or(false) {
                    style = style.add_modifier(Modifier::BOLD);
                }
                if cfg.italic.unwrap_or(false) {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if cfg.underlined.unwrap_or(false) {
                    style = style.add_modifier(Modifier::UNDERLINED);
                }
                if cfg.crossed_out.unwrap_or(false) {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }
                if cfg.dim.unwrap_or(false) {
                    style = style.add_modifier(Modifier::DIM);
                }
                (tag, style)
            })
            .collect()
    }

    pub fn get_styles() -> &'static HashMap<String, Style> {
        &CONFIG.styles
    }

    pub fn get_selectors() -> &'static HashMap<String, String> {
        &CONFIG.selectors
    }

    pub fn get_ignored_tags() -> &'static HashSet<String> { &CONFIG.ignored_tags }

    pub fn get_block_elements() -> &'static HashSet<String> { &CONFIG.block_elements }
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
        _ => Color::Reset,
    }
}

