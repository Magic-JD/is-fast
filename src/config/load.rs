use crate::cli::command::{CacheMode, ColorMode};
use crate::config::files::config_location;
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
use once_cell::sync::{Lazy, OnceCell};
use ratatui::style::Style;
use scraper::ElementRef;
use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fs;
use toml;

static CONFIG: OnceCell<Config> = OnceCell::new();
pub const DEFAULT_CONFIG: &str = include_str!("config.toml");
pub const ALTERNATE_USERAGENT: &str = include_str!("alternate_useragent.toml");

static EMBEDDED_CONFIG_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "alternate_useragent.toml".to_string(),
        ALTERNATE_USERAGENT.to_string(),
    );
    map
});
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

#[derive(Debug, Clone, Default)]
struct TagIdentifier {
    unconditional: bool,
    classes: HashSet<String>,
    ids: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
struct SitePicker {
    sites: HashMap<String, SiteConfig>,
    matcher: GlobSet,
    globs: Vec<Glob>,
}

impl SitePicker {
    fn get_match(&self, url: &str) -> &str {
        self.matcher
            .matches(url)
            .iter()
            .find_map(|idx| self.globs.get(*idx))
            .map(|glob| glob.glob())
            .unwrap_or("")
    }

    pub fn get_format_config(&self, url: &str) -> &FormatConfig {
        &self
            .sites
            .get(self.get_match(url))
            .expect("should be either empty string and default or found")
            .format
    }

    pub fn get_call_config(&self, url: &str) -> &CallConfig {
        &self
            .sites
            .get(self.get_match(url))
            .expect("should be either empty string and default or found")
            .call
    }
}

#[derive(Debug, Clone, Default)]
struct SiteConfig {
    format: FormatConfig,
    call: CallConfig,
}

#[derive(Debug, Clone, Default)]
struct CallConfig {
    headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    ignored_tags: HashMap<String, TagIdentifier>,
    block_elements: HashMap<String, TagIdentifier>,
    tag_styles: HashMap<String, Style>,
}

impl FormatConfig {
    pub fn new(
        ignored_tags: HashSet<String>,
        block_elements: HashSet<String>,
        tag_styles: HashMap<String, Style>,
    ) -> Self {
        let ignored_tags_map = Self::build_map_from_selectors(ignored_tags);
        let block_elements_map = Self::build_map_from_selectors(block_elements);
        Self {
            ignored_tags: ignored_tags_map,
            block_elements: block_elements_map,
            tag_styles,
        }
    }

    fn build_map_from_selectors(ignored_tags: HashSet<String>) -> HashMap<String, TagIdentifier> {
        let mut ignored_tags_map: HashMap<String, TagIdentifier> = HashMap::new();
        for tag in ignored_tags {
            let mut class_split = tag.split('.');
            let tag = class_split.next().unwrap_or_else(|| {
                log::error!("Invalid css selector - must be of the format TAG#ID.CLASS, {tag}");
                ""
            });
            let classes = class_split.collect::<Vec<&str>>();
            let mut id_split = tag.split('#');
            let tag = id_split.next().unwrap_or_else(|| {
                log::error!("Invalid css selector - must be of the format TAG#ID.CLASS, {tag}");
                ""
            });
            let id = id_split.next();
            let tag_identifier = ignored_tags_map.entry(tag.to_string()).or_default();
            if classes.is_empty() && id.is_none() {
                tag_identifier.unconditional = true;
            }
            tag_identifier
                .classes
                .extend(classes.into_iter().map(String::from));
            if let Some(id) = id {
                tag_identifier.ids.insert(id.to_string());
            }
        }
        ignored_tags_map
    }

    pub fn is_element_ignored(&self, element: &ElementRef) -> bool {
        let tag = element.value().name();
        let tag_identifier = self.ignored_tags.get(tag);
        let general_identifier = self.ignored_tags.get("");
        Self::matches_tag(element, tag_identifier) || Self::matches_tag(element, general_identifier)
    }

    fn matches_tag(element: &ElementRef, tag_identifier: Option<&TagIdentifier>) -> bool {
        match tag_identifier {
            Some(tag_identifier) => {
                tag_identifier.unconditional
                    || Self::element_contains_class(element, tag_identifier)
                    || Self::element_contains_id(element, tag_identifier)
            }
            None => false,
        }
    }

    fn element_contains_id(element: &ElementRef, tag_identifier: &TagIdentifier) -> bool {
        element
            .value()
            .id()
            .is_some_and(|id| tag_identifier.ids.contains(id))
    }

    fn element_contains_class(element: &ElementRef, tag_identifier: &TagIdentifier) -> bool {
        element
            .value()
            .attr("class")
            .into_iter()
            .flat_map(|class_attr| class_attr.split_whitespace())
            .any(|class| tag_identifier.classes.contains(class))
    }

    pub fn is_block_element(&self, element: &ElementRef) -> bool {
        let tag_identifier = self.block_elements.get(element.value().name());
        let general_identifier = self.block_elements.get("");
        Self::matches_tag(element, tag_identifier) || Self::matches_tag(element, general_identifier)
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
        let mut config: RawConfig = toml::from_str(DEFAULT_CONFIG)
            .map_err(|e| println!("{e}"))
            .unwrap_or(RawConfig::default());
        _ = get_user_specified_config().map(|u_config| override_defaults(&mut config, u_config));
        let format = Self::create_format_config(&config, ignored_additional, no_block);
        let extraction = Self::create_extraction_config(
            args_color_mode,
            selector_override,
            nth_element,
            &config,
        );
        let cache = Self::create_cache_config(cache_mode, &config);
        let history_widget = Self::create_history_widget_config(&config);

        let mut site_mapping = HashMap::new();
        for (url, filename) in config.custom_config.clone() {
            let custom = Self::get_custom_config(&filename);
            let site_config =
                Self::update_current_with_custom(config.headers.clone(), &format, custom);

            site_mapping.insert(url, site_config);
        }
        site_mapping.insert(
            "".to_string(),
            SiteConfig {
                format,
                call: CallConfig {
                    headers: config.headers.clone(),
                },
            },
        );
        let (matcher, globs) = generate_globs(config.custom_config.keys().collect());
        let site_picker = SitePicker {
            sites: site_mapping,
            matcher,
            globs,
        };
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
            open_tool: config
                .misc
                .as_ref()
                .and_then(|misc| misc.open_tool.clone())
                .clone(),
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
            history_widget,
            sites: site_picker,
        }
    }

    fn update_current_with_custom(
        mut headers: HashMap<String, String>,
        current_format: &FormatConfig,
        custom: RawConfig,
    ) -> SiteConfig {
        let mut current_update = current_format.clone();
        let updated_format = Self::create_format_config(&custom, vec![], false);
        updated_format
            .tag_styles
            .into_iter()
            .for_each(|(key, style)| {
                current_update.tag_styles.insert(key, style);
            });
        updated_format
            .block_elements
            .into_iter()
            .for_each(|(key, ti)| {
                current_update.block_elements.insert(key, ti);
            });
        updated_format
            .ignored_tags
            .into_iter()
            .for_each(|(key, ti)| {
                current_update.ignored_tags.insert(key, ti);
            });
        for (k, v) in custom.headers.into_iter() {
            headers.insert(k, v);
        }
        SiteConfig {
            format: current_update,
            call: CallConfig { headers },
        }
    }

    fn get_custom_config(filename: &String) -> RawConfig {
        let mut path = config_location();
        path.push(filename);
        let site_config = fs::read_to_string(path)
            .ok()
            .or_else(|| EMBEDDED_CONFIG_MAP.get(filename).cloned())
            .expect("Should have found embedded or locally");
        let custom: RawConfig = toml::from_str(site_config.as_str())
            .map_err(|e| println!("{e}"))
            .unwrap_or(RawConfig::default());
        custom
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

    fn create_format_config(
        config: &RawConfig,
        ignored_additional: Vec<String>,
        no_block: bool,
    ) -> FormatConfig {
        let mut ignored_tags: HashSet<String> = config
            .format
            .as_ref()
            .map(|format| format.ignored_tags.iter().cloned().collect())
            .unwrap_or_default();
        ignored_tags.extend(ignored_additional);
        let block_elements = if no_block {
            HashSet::new()
        } else {
            config
                .format
                .as_ref()
                .map(|format| format.block_elements.iter().cloned().collect())
                .unwrap_or_default()
        };
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

    pub fn get_format_config(url: &str) -> FormatConfig {
        Self::get_config().sites.get_format_config(url).clone()
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

    pub fn get_headers(url: &str) -> &'static HashMap<String, String> {
        &Self::get_config().sites.get_call_config(url).headers
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
