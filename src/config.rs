use once_cell::sync::Lazy;
use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use toml;

static CONFIG: Lazy<Config> = Lazy::new(Config::load);

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
struct RawConfig {
    #[serde(default)]
    styles: HashMap<String, TagStyleConfig>,
    #[serde(default)]
    selectors: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Config {
    styles: HashMap<String, Style>,
    selectors: HashMap<String, String>,
}

impl Config {
    fn load() -> Self {
        let user_config = dirs::config_dir()
            .map(|p| p.join("is-fast/config.toml"))
            .and_then(|path| fs::read_to_string(&path).ok())
            .and_then(|content| toml::from_str::<RawConfig>(&content).ok());

        let mut config: RawConfig = toml::from_str(DEFAULT_CONFIG).unwrap_or(RawConfig {
            styles: HashMap::new(),
            selectors: HashMap::new(),
        });

        if let Some(u_config) = user_config {
            for (tag, user_style) in u_config.styles {
                config.styles.insert(tag, user_style);
            }
            for (site, selector) in u_config.selectors {
                config.selectors.insert(site, selector);
            }
        }

        Self {
            styles: Self::convert_styles(config.styles),
            selectors: config.selectors,
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

const DEFAULT_CONFIG: &str = include_str!("config/default_config.toml");
