use ratatui::style::{Color, Modifier, Style};
use serde::Deserialize;
use std::{collections::HashMap, fs};
use toml;

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
struct Config {
    styles: HashMap<String, TagStyleConfig>,
}

pub fn load_config() -> HashMap<String, Style> {
    let user_config = dirs::config_dir()
        .map(|p| p.join("is-fast/config.toml"))
        .and_then(|path| fs::read_to_string(&path).ok())
        .and_then(|content| toml::from_str::<Config>(&content).ok());

    let mut config: Config = toml::from_str(DEFAULT_CONFIG)
        .unwrap_or(Config { styles: HashMap::new()});

    user_config.map(|u_config| {
        for (tag, user_style) in u_config.styles {
            config.styles.insert(tag, user_style);
        }
    });
    convert_config(config)

}

fn convert_config(config: Config) -> HashMap<String, Style> {
    config
        .styles
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

const DEFAULT_CONFIG: &str = r#"
[styles.h1]
bold = true

[styles.h2]
bold = true

[styles.h3]
bold = true

[styles.a]
fg = "Cyan"

[styles.code]
fg = "Red"

[styles.em]
italic = true

[styles.i]
italic = true

[styles.strong]
bold = true

[styles.b]
bold = true

[styles.blockquote]
fg = "Gray"
italic = true

[styles.del]
crossed_out = true

[styles.ins]
underlined = true

[styles.mark]
fg = "Black"
bg = "Yellow"

[styles.small]
fg = "Gray"

[styles.sub]
fg = "Gray"
dim = true

[styles.sup]
fg = "Gray"
dim = true

[styles.pre]
fg = "White"
bg = "Black"

[styles.kbd]
fg = "White"
bg = "DarkGray"

[styles.var]
fg = "Cyan"

[styles.samp]
fg = "Magenta"

[styles.u]
underlined = true

[styles.li]
bold = true

[styles.dt]
bold = true

[styles.dd]
fg = "Gray"
"#;
