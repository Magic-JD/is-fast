use crate::cli::command::CacheMode;
use crate::config::color_conversion::Style;
use crate::config::files::config_location;
use crate::config::format::FormatConfig;
use crate::config::glob_generation::generate_globs;
use crate::config::load::{get_user_base_config_file, DEFAULT_CONFIG};
use crate::config::site_raw::{override_defaults_site, CacheSection, SiteRawConfig};
use crate::search_engine::cache::CacheConfig;
use globset::{Glob, GlobSet};
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};
use std::fs;

pub const ALTERNATE_HEADERS: &str = include_str!("alternate_headers.toml");

static EMBEDDED_CONFIG_MAP: Lazy<HashMap<String, String>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(
        "alternate_headers.toml".to_string(),
        ALTERNATE_HEADERS.to_string(),
    );
    map
});
const MS_IN_SECOND: i64 = 1000;

#[derive(Debug, Clone, Default)]
pub struct SitePicker {
    sites: HashMap<String, SiteConfig>,
    matcher: GlobSet,
    globs: Vec<Glob>,
}

#[derive(Debug, Clone, Default)]
pub struct SiteConfig {
    pub(crate) format: FormatConfig,
    pub(crate) call: CallConfig,
    pub(crate) cache: CacheConfig,
    pub(crate) syntax: SyntaxConfig,
}

impl SiteConfig {
    pub fn get_format(&self) -> &FormatConfig {
        &self.format
    }
    pub fn get_call(&self) -> &CallConfig {
        &self.call
    }
    pub fn get_cache(&self) -> &CacheConfig {
        &self.cache
    }
    pub fn get_syntax(&self) -> &SyntaxConfig {
        &self.syntax
    }
}

#[derive(Debug, Clone, Default)]
pub struct CallConfig {
    headers: HashMap<String, String>,
}

impl CallConfig {
    pub fn get_headers(&self) -> &HashMap<String, String> {
        &self.headers
    }
}

#[derive(Debug, Clone, Default)]
pub struct SyntaxConfig {
    pub syntax_default_language: String,
    pub syntax_highlighting_theme: String,
}

impl SyntaxConfig {
    pub fn get_syntax_default_language(&self) -> &str {
        &self.syntax_default_language
    }

    pub fn get_syntax_highlighting_theme(&self) -> &str {
        &self.syntax_highlighting_theme
    }
}

impl SitePicker {
    pub fn new(
        custom_configs: &HashMap<String, Vec<String>>,
        ignored_additional: &[String],
        no_block: bool,
        cache_mode: Option<&CacheMode>,
        styles: &[(String, Style)],
    ) -> Self {
        let mut site: SiteRawConfig = toml::from_str(DEFAULT_CONFIG)
            .map_err(|e| println!("{e}"))
            .unwrap_or(SiteRawConfig::default());
        _ = get_user_specified_site_config()
            .map(|u_config| override_defaults_site(&mut site, u_config));

        let base_site_config =
            Self::create_base_site_config(&site, ignored_additional, no_block, cache_mode, styles);
        let mut site_mapping = HashMap::new();
        site_mapping.insert(String::new(), base_site_config);
        for (url, filenames) in custom_configs {
            let mut base_site_raw_mut = site.clone();
            for file in filenames {
                let custom = Self::get_custom_config(file);
                override_defaults_site(&mut base_site_raw_mut, custom);
            }
            site_mapping.insert(
                url.to_string(),
                Self::create_base_site_config(
                    &base_site_raw_mut,
                    ignored_additional,
                    no_block,
                    cache_mode,
                    styles,
                ),
            );
        }
        let (matcher, globs) = generate_globs(custom_configs.keys().collect());
        SitePicker {
            sites: site_mapping,
            matcher,
            globs,
        }
    }

    fn create_base_site_config(
        raw: &SiteRawConfig,
        ignored_additional: &[String],
        no_block: bool,
        cache_mode: Option<&CacheMode>,
        styles: &[(String, Style)],
    ) -> SiteConfig {
        let format = Self::create_format_config(raw, ignored_additional, no_block, styles);
        let cache = Self::create_cache_config(cache_mode, raw);
        let syntax = Self::create_syntax_config(raw);
        let call = Self::create_call_config(raw);
        SiteConfig {
            format,
            call,
            cache,
            syntax,
        }
    }

    fn create_call_config(raw: &SiteRawConfig) -> CallConfig {
        CallConfig {
            headers: raw.headers.clone(),
        }
    }

    fn create_syntax_config(raw: &SiteRawConfig) -> SyntaxConfig {
        let syntax_default_language = raw
            .syntax
            .as_ref()
            .and_then(|syntax| syntax.default_language.clone())
            .unwrap_or_default();
        let syntax_highlighting_theme = raw
            .syntax
            .as_ref()
            .and_then(|syntax| syntax.theme.clone())
            .unwrap_or_default();
        SyntaxConfig {
            syntax_default_language,
            syntax_highlighting_theme,
        }
    }

    fn create_format_config(
        config: &SiteRawConfig,
        ignored_additional: &[String],
        no_block: bool,
        styles: &[(String, Style)],
    ) -> FormatConfig {
        let mut ignored_tags: HashSet<String> = config
            .format
            .as_ref()
            .map(|format| format.ignored_tags.iter().cloned().collect())
            .unwrap_or_default();
        ignored_tags.extend(ignored_additional.iter().cloned());
        let block_elements = if no_block {
            HashSet::new()
        } else {
            config
                .format
                .as_ref()
                .map(|format| format.block_elements.iter().cloned().collect())
                .unwrap_or_default()
        };
        let indent_elements = config
            .format
            .as_ref()
            .map(|format| format.indent_elements.iter().cloned().collect())
            .unwrap_or_default();
        let mut existing_styles = config.styles.clone();
        for (key, value) in styles {
            let new = existing_styles.get(key).map_or(*value, |s| s.patch(value));
            existing_styles.insert(key.to_string(), new);
        }
        FormatConfig::new(
            ignored_tags,
            block_elements,
            indent_elements,
            existing_styles.clone(),
        )
    }

    fn create_cache_config(cache_mode: Option<&CacheMode>, config: &SiteRawConfig) -> CacheConfig {
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
    fn get_match(&self, url: &str) -> &str {
        self.matcher
            .matches(url)
            .iter()
            .filter_map(|&idx| self.globs.get(idx))
            // Take the longest possible match
            .max_by_key(|glob| glob.glob().len())
            .map_or("", |glob| glob.glob())
    }

    fn get_custom_config(filename: &String) -> SiteRawConfig {
        let mut path = config_location();
        path.push(filename);
        let site_config = fs::read_to_string(path)
            .ok()
            .or_else(|| EMBEDDED_CONFIG_MAP.get(filename).cloned())
            .expect("Should have found embedded or locally");
        toml::from_str(site_config.as_str())
            .map_err(|e| println!("{e}"))
            .unwrap_or(SiteRawConfig::default())
    }

    pub fn get_site_config(&self, url: &str) -> &SiteConfig {
        self.sites
            .get(self.get_match(url))
            .expect("Should have found embedded or locally")
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
fn get_user_specified_site_config() -> Option<SiteRawConfig> {
    get_user_base_config_file().and_then(|str| toml::from_str(&str).ok())
}
