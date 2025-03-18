use crate::cli::command::{CacheMode, ColorMode};
use crate::config::raw::{
    convert_styles, generate_globs, get_user_specified_config, override_defaults, parse_color,
    CacheSection, RawConfig,
};
use crate::search_engine::cache::CacheConfig;
use crate::search_engine::search_type::SearchEngine;
use crate::search_engine::search_type::SearchEngine::{DuckDuckGo, Google, Kagi};
use crate::DisplayConfig;
use globset::{Glob, GlobSet};
use nucleo_matcher::pattern::AtomKind;
use once_cell::sync::OnceCell;
use ratatui::style::Style;
use std::collections::{HashMap, HashSet};
use toml;

static CONFIG: OnceCell<Config> = OnceCell::new();
pub const DEFAULT_CONFIG_LOCATION: &str = include_str!("config.toml");
const MS_IN_SECOND: i64 = 1000;

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
pub struct FormatConfig {
    ignored_tags: HashSet<String>,
    block_elements: HashSet<String>,
    tag_styles: HashMap<String, Style>,
}

impl FormatConfig {
    pub fn new(
        ignored_tags: HashSet<String>,
        block_elements: HashSet<String>,
        tag_styles: HashMap<String, Style>,
    ) -> Self {
        Self {
            ignored_tags,
            block_elements,
            tag_styles,
        }
    }

    pub fn is_tag_ignored(&self, tag: &str) -> bool {
        self.ignored_tags.contains(tag)
    }

    pub fn is_block_element(&self, tag: &str) -> bool {
        self.block_elements.contains(tag)
    }

    pub fn style_for_tag(&self, tag: &str) -> Option<&Style> {
        self.tag_styles.get(tag)
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
    syntax_default_language: String,
    syntax_highlighting_theme: String,
    page_margin: u16,
    border_color: Style,
    search_type: AtomKind,
    search_engine: SearchEngine,
    open_tool: Option<String>,
    scroll: Scroll,
    history_enabled: bool,
    cache: CacheConfig,
    pretty_print: Vec<DisplayConfig>,
    extraction: ExtractionConfig,
    format: FormatConfig,
    history_widget: HistoryWidgetConfig,
}

impl Config {
    pub fn init(
        args_color_mode: Option<ColorMode>,
        cache_command: Option<&CacheMode>,
        no_history: bool,
        pretty_print: Vec<DisplayConfig>,
        selector_override: Option<String>,
        nth_element: Vec<usize>,
    ) {
        let this = Self::new(
            args_color_mode,
            cache_command,
            no_history,
            pretty_print,
            selector_override,
            nth_element,
        );
        CONFIG.try_insert(this).expect("Failed to insert config");
    }

    fn default() -> Config {
        Self::new(None, None, false, vec![], None, vec![])
    }

    fn new(
        args_color_mode: Option<ColorMode>,
        cache_mode: Option<&CacheMode>,
        no_history: bool,
        pretty_print: Vec<DisplayConfig>,
        selector_override: Option<String>,
        nth_element: Vec<usize>,
    ) -> Self {
        let mut config: RawConfig = toml::from_str(DEFAULT_CONFIG_LOCATION)
            .map_err(|e| println!("{e}"))
            .unwrap_or(RawConfig::default());
        _ = get_user_specified_config().map(|u_config| override_defaults(&mut config, u_config));
        let format = Self::create_format_config(&config);
        let extraction = Self::create_extraction_config(
            args_color_mode,
            selector_override,
            nth_element,
            &config,
        );
        let cache = Self::create_cache_config(cache_mode, &config);
        let history_widget = Self::create_history_widget_config(&config);
        Self {
            syntax_default_language: config
                .syntax
                .as_ref()
                .and_then(|syntax| syntax.default_language.clone())
                .unwrap_or_default(),
            syntax_highlighting_theme: config
                .syntax
                .as_ref()
                .and_then(|syntax| syntax.theme.clone())
                .unwrap_or_default(),
            page_margin: config
                .display
                .as_ref()
                .and_then(|display| display.page_margin)
                .unwrap_or_default(),
            border_color: config
                .display
                .as_ref()
                .and_then(|display| display.border_color.clone())
                .map(|color| Style::new().fg(parse_color(&color)))
                .unwrap_or_default(),
            search_type: to_atom_kind(
                &config
                    .history
                    .as_ref()
                    .and_then(|history| history.search_type.clone())
                    .unwrap_or_default(),
            ),
            search_engine: to_search_engine(
                &config
                    .search
                    .as_ref()
                    .and_then(|search| search.engine.clone())
                    .unwrap_or_default(),
            ),
            open_tool: config.misc.and_then(|misc| misc.open_tool).clone(),
            scroll: convert_to_scroll(
                &config
                    .display
                    .as_ref()
                    .and_then(|display| display.scroll.clone())
                    .unwrap_or_default(),
            ),
            history_enabled: if no_history {
                false
            } else {
                config
                    .history
                    .as_ref()
                    .and_then(|history| history.enabled)
                    .unwrap_or(true)
            },
            cache,
            pretty_print,
            extraction,
            format,
            history_widget,
        }
    }

    fn create_history_widget_config(config: &RawConfig) -> HistoryWidgetConfig {
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
        config: &RawConfig,
    ) -> ExtractionConfig {
        let (matcher, globs) = generate_globs(config);
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

    fn create_format_config(config: &RawConfig) -> FormatConfig {
        let ignored_tags = config
            .format
            .as_ref()
            .map(|format| format.ignored_tags.iter().cloned().collect())
            .unwrap_or_default();
        let block_elements = config
            .format
            .as_ref()
            .map(|format| format.block_elements.iter().cloned().collect())
            .unwrap_or_default();
        let tag_styles = convert_styles(&config.styles);
        FormatConfig::new(ignored_tags, block_elements, tag_styles)
    }

    fn create_cache_config(cache_mode: Option<&CacheMode>, config: &RawConfig) -> CacheConfig {
        if let Some(CacheMode::Flash) = cache_mode {
            CacheConfig::new(CacheMode::ReadWrite, usize::MAX, 5 * MS_IN_SECOND, 0)
        } else {
            Self::extract_cache_from_raw(cache_mode, config.cache.as_ref())
        }
    }

    fn extract_cache_from_raw(
        cache_mode: Option<&CacheMode>,
        config: Option<&CacheSection>,
    ) -> CacheConfig {
        config.map_or_else(
            || {
                CacheConfig::new(
                    cache_mode.cloned().unwrap_or(CacheMode::Never),
                    100,
                    300 * MS_IN_SECOND,
                    10,
                )
            },
            |cache_section| {
                CacheConfig::new(
                    cache_mode
                        .cloned()
                        .or_else(|| {
                            cache_section
                                .cache_mode
                                .as_deref()
                                .map(convert_to_cache_mode)
                        })
                        .unwrap_or(CacheMode::Never),
                    cache_section.max_size.unwrap_or(100),
                    cache_section.ttl.unwrap_or(300) * MS_IN_SECOND,
                    10,
                )
            },
        )
    }

    fn get_config() -> &'static Config {
        CONFIG.get_or_init(Config::default)
    }

    pub fn get_format_config() -> FormatConfig {
        Self::get_config().format.clone()
    }

    pub fn get_default_language() -> &'static String {
        &Self::get_config().syntax_default_language
    }

    pub fn get_syntax_highlighting_theme() -> &'static String {
        &Self::get_config().syntax_highlighting_theme
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

    pub fn get_cache_config() -> CacheConfig {
        Self::get_config().cache.clone()
    }

    pub fn get_extractor_config() -> ExtractionConfig {
        Self::get_config().extraction.clone()
    }

    pub fn get_pretty_print() -> &'static [DisplayConfig] {
        &Self::get_config().pretty_print
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

fn convert_to_cache_mode(cache_mode: &str) -> CacheMode {
    match cache_mode.to_lowercase().as_str() {
        "disabled" => CacheMode::Never,
        "readwrite" => CacheMode::ReadWrite,
        "read" => CacheMode::Read,
        "write" => CacheMode::Write,
        _ => CacheMode::Never,
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

#[derive(Debug)]
pub enum Scroll {
    Full,
    Half,
    Discrete(u16),
}
