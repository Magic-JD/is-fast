use crate::cli::command::{CacheMode, ColorMode};
use crate::config::color_conversion::parse_color;
use crate::config::files::config_path;
use crate::config::glob_generation::generate_globs;
use crate::config::site::{SiteConfig, SitePicker};
use crate::config::tool_raw::{override_defaults_tool, ToolRawConfig};
use crate::search_engine::search_type::SearchEngine;
use crate::search_engine::search_type::SearchEngine::{DuckDuckGo, Google, Kagi};
use crate::DisplayConfig;
use globset::{Glob, GlobSet};
use nucleo_matcher::pattern::AtomKind;
use once_cell::sync::OnceCell;
use ratatui::style::Style;
use std::collections::HashMap;
use std::fs;
use toml;

static CONFIG: OnceCell<Config> = OnceCell::new();
pub const DEFAULT_CONFIG: &str = include_str!("config.toml");

#[derive(Debug, Clone)]
pub struct HistoryWidgetConfig {
    url: Style,
    title: Style,
    time: Style,
    text: Style,
}

impl HistoryWidgetConfig {
    pub fn new(url: Style, title: Style, time: Style, text: Style) -> Self {
        Self {
            url,
            title,
            time,
            text,
        }
    }

    pub fn get_url_style(&self) -> &Style {
        &self.url
    }

    pub fn get_title_style(&self) -> &Style {
        &self.title
    }

    pub fn get_time_style(&self) -> &Style {
        &self.time
    }

    pub fn get_text_style(&self) -> &Style {
        &self.text
    }
}

#[derive(Debug, Clone)]
pub struct ExtractionConfig {
    color_mode: ColorMode,
    nth_element: Vec<usize>,
    selectors: HashMap<String, String>,
    selector_override: Option<String>,
    matcher: GlobSet,
    globs: Vec<Glob>,
}

impl ExtractionConfig {
    pub fn new(
        color_mode: ColorMode,
        nth_element: Vec<usize>,
        selectors: HashMap<String, String>,
        selector_override: Option<String>,
        matcher: GlobSet,
        globs: Vec<Glob>,
    ) -> Self {
        Self {
            color_mode,
            nth_element,
            selectors,
            selector_override,
            matcher,
            globs,
        }
    }

    pub fn color_mode(&self) -> &ColorMode {
        &self.color_mode
    }

    pub fn get_selectors(&self, url: &str) -> &str {
        self.selector_override
            .as_ref()
            .or_else(|| {
                self.matcher
                    .matches(url)
                    .iter()
                    .find_map(|idx| self.globs.get(*idx))
                    .and_then(|glob| self.selectors.get(&glob.to_string()))
            })
            .map_or_else(|| "body", String::as_str)
    }

    pub fn nth_element(&self) -> &Vec<usize> {
        &self.nth_element
    }
}

#[derive(Debug)]
pub struct Config {
    page_margin: u16,
    border_color: Style,
    search_type: AtomKind,
    search_engine: SearchEngine,
    open_tool: Option<String>,
    scroll: Scroll,
    history_enabled: bool,
    pretty_print: Vec<DisplayConfig>,
    extraction: ExtractionConfig,
    history_widget: HistoryWidgetConfig,
    sites: SitePicker,
}

impl Config {
    // This is where the key configuration is combined, and I would rather have these values being passed
    // in individually rather than just passing in the arg or combining them into an arbitrary object to appease
    // clippy. If this grows any larger I will revisit and rework.
    #[allow(clippy::too_many_arguments)]
    pub fn init(
        args_color_mode: Option<ColorMode>,
        cache_command: Option<&CacheMode>,
        no_history: bool,
        pretty_print: Vec<DisplayConfig>,
        selector_override: Option<String>,
        ignored_additional: Vec<String>,
        no_block: bool,
        nth_element: Vec<usize>,
    ) {
        let this = Self::new(
            args_color_mode,
            cache_command,
            no_history,
            pretty_print,
            selector_override,
            ignored_additional,
            no_block,
            nth_element,
        );
        CONFIG.try_insert(this).expect("Failed to insert config");
    }

    fn default() -> Config {
        Self::new(None, None, false, vec![], None, vec![], false, vec![])
    }

    // This is where the key configuration is combined, and I would rather have these values being passed
    // in individually rather than just passing in the arg or combining them into an arbitrary object to appease
    // clippy. If this grows any larger I will revisit and rework.
    #[allow(clippy::too_many_arguments)]
    fn new(
        args_color_mode: Option<ColorMode>,
        cache_mode: Option<&CacheMode>,
        no_history: bool,
        pretty_print: Vec<DisplayConfig>,
        selector_override: Option<String>,
        ignored_additional: Vec<String>,
        no_block: bool,
        nth_element: Vec<usize>,
    ) -> Self {
        let mut tool: ToolRawConfig =
            toml::from_str(DEFAULT_CONFIG).unwrap_or(ToolRawConfig::default());
        _ = get_user_specified_tool_config()
            .map(|u_config| override_defaults_tool(&mut tool, u_config));
        let site_picker = SitePicker::new(
            &tool.custom_config,
            ignored_additional,
            no_block,
            cache_mode,
        );
        let extraction =
            Self::create_extraction_config(args_color_mode, selector_override, nth_element, &tool);
        let history_widget = Self::create_history_widget_config(&tool);

        Self {
            page_margin: tool
                .display
                .as_ref()
                .and_then(|display| display.page_margin)
                .unwrap_or_default(),
            border_color: tool
                .display
                .as_ref()
                .and_then(|display| display.border_color.clone())
                .map(|color| Style::new().fg(parse_color(&color)))
                .unwrap_or_default(),
            search_type: to_atom_kind(
                &tool
                    .history
                    .as_ref()
                    .and_then(|history| history.search_type.clone())
                    .unwrap_or_default(),
            ),
            search_engine: to_search_engine(
                &tool
                    .search
                    .as_ref()
                    .and_then(|search| search.engine.clone())
                    .unwrap_or_default(),
            ),
            open_tool: tool
                .misc
                .as_ref()
                .and_then(|misc| misc.open_tool.clone())
                .clone(),
            scroll: convert_to_scroll(
                &tool
                    .display
                    .as_ref()
                    .and_then(|display| display.scroll.clone())
                    .unwrap_or_default(),
            ),
            history_enabled: if no_history {
                false
            } else {
                tool.history
                    .as_ref()
                    .and_then(|history| history.enabled)
                    .unwrap_or(true)
            },
            pretty_print,
            extraction,
            history_widget,
            sites: site_picker,
        }
    }

    fn create_history_widget_config(config: &ToolRawConfig) -> HistoryWidgetConfig {
        let title_style = config
            .history
            .as_ref()
            .and_then(|history| history.title_color.clone())
            .map(|color| Style::new().fg(parse_color(&color)))
            .unwrap_or_default();
        let url_style = config
            .history
            .as_ref()
            .and_then(|history| history.url_color.clone())
            .map(|color| Style::new().fg(parse_color(&color)))
            .unwrap_or_default();
        let time_style = config
            .history
            .as_ref()
            .and_then(|history| history.time_color.clone())
            .map(|color| Style::new().fg(parse_color(&color)))
            .unwrap_or_default();
        let text_style = config
            .history
            .as_ref()
            .and_then(|history| history.text_color.clone())
            .map(|color| Style::new().fg(parse_color(&color)))
            .unwrap_or_default();
        HistoryWidgetConfig::new(url_style, title_style, time_style, text_style)
    }

    fn create_extraction_config(
        args_color_mode: Option<ColorMode>,
        selector_override: Option<String>,
        nth_element: Vec<usize>,
        config: &ToolRawConfig,
    ) -> ExtractionConfig {
        let (matcher, globs) = generate_globs(config.selectors.keys().collect());
        let selectors = config.selectors.clone();
        let color_mode = args_color_mode.unwrap_or_else(|| {
            convert_to_color_mode(
                &config
                    .display
                    .as_ref()
                    .and_then(|display| display.color_mode.clone())
                    .unwrap_or_default(),
            )
        });
        ExtractionConfig::new(
            color_mode,
            nth_element,
            selectors,
            selector_override,
            matcher,
            globs,
        )
    }

    fn get_config() -> &'static Config {
        CONFIG.get_or_init(Config::default)
    }

    pub fn get_page_margin() -> u16 {
        Self::get_config().page_margin
    }

    pub fn get_border_color() -> &'static Style {
        &Self::get_config().border_color
    }

    pub fn get_history_widget_config() -> HistoryWidgetConfig {
        Self::get_config().history_widget.clone()
    }

    pub fn get_search_type() -> &'static AtomKind {
        &Self::get_config().search_type
    }
    pub fn get_search_engine() -> &'static SearchEngine {
        &Self::get_config().search_engine
    }

    pub fn get_open_command() -> Option<&'static String> {
        Self::get_config().open_tool.as_ref()
    }

    pub fn get_scroll() -> &'static Scroll {
        &Self::get_config().scroll
    }

    pub fn get_history_enabled() -> &'static bool {
        &Self::get_config().history_enabled
    }

    pub fn get_extractor_config() -> ExtractionConfig {
        Self::get_config().extraction.clone()
    }

    pub fn get_pretty_print() -> &'static [DisplayConfig] {
        &Self::get_config().pretty_print
    }

    pub fn get_site_config(url: &str) -> &SiteConfig {
        Self::get_config().sites.get_site_config(url)
    }
}

fn convert_to_color_mode(color_mode: &str) -> ColorMode {
    match color_mode.to_lowercase().as_str() {
        "tui" => ColorMode::Tui,
        "always" => ColorMode::Always,
        "never" => ColorMode::Never,
        _ => ColorMode::Tui,
    }
}

fn convert_to_scroll(scroll: &str) -> Scroll {
    match scroll.to_lowercase().as_str() {
        "full" => Scroll::Full,
        "half" => Scroll::Half,
        num_str if num_str.parse::<u16>().is_ok() => {
            Scroll::Discrete(num_str.parse().unwrap_or_default())
        }
        _ => Scroll::Full,
    }
}

fn to_atom_kind(search_type: &str) -> AtomKind {
    match search_type.to_lowercase().as_str() {
        "fuzzy" => AtomKind::Fuzzy,
        "exact" => AtomKind::Exact,
        "substring" => AtomKind::Substring,
        _ => AtomKind::Fuzzy,
    }
}

fn to_search_engine(search_engine: &str) -> SearchEngine {
    match search_engine.to_lowercase().as_str() {
        "duckduckgo" => DuckDuckGo,
        "google" => Google,
        "kagi" => Kagi,
        _ => DuckDuckGo, // Default to duckduckgo
    }
}

fn get_user_specified_tool_config() -> Option<ToolRawConfig> {
    get_user_base_config_file().and_then(|str| toml::from_str(&str).ok())
}

pub fn get_user_base_config_file() -> Option<String> {
    let buff = config_path();
    fs::read_to_string(buff).ok()
}

#[derive(Debug)]
pub enum Scroll {
    Full,
    Half,
    Discrete(u16),
}
