use crate::config::color_conversion::TagStyleConfig;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
pub struct FormatSection {
    #[serde(default)]
    pub(crate) ignored_tags: Vec<String>,
    #[serde(default)]
    pub(crate) block_elements: Vec<String>,
    #[serde(default)]
    pub(crate) indent_elements: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SyntaxHighlightingSection {
    #[serde(default)]
    pub(crate) theme: Option<String>,
    #[serde(default)]
    pub(crate) default_language: Option<String>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct SiteRawConfig {
    #[serde(default)]
    pub(crate) styles: HashMap<String, TagStyleConfig>,
    #[serde(default)]
    pub(crate) format: Option<FormatSection>,
    #[serde(default)]
    pub(crate) syntax: Option<SyntaxHighlightingSection>,
    #[serde(default)]
    pub(crate) cache: Option<CacheSection>,
    #[serde(default)]
    pub(crate) headers: HashMap<String, String>,
}

impl SiteRawConfig {
    pub fn default() -> Self {
        Self {
            styles: HashMap::new(),
            format: None,
            syntax: None,
            cache: None,
            headers: HashMap::new(),
        }
    }
}

pub fn override_defaults_site(config: &mut SiteRawConfig, mut u_config: SiteRawConfig) {
    for (tag, user_style) in u_config.styles {
        config.styles.insert(tag, user_style);
    }
    config.format = Some(override_format(
        config.format.take(),
        u_config.format.take(),
    ));
    config.syntax = Some(override_syntax(
        config.syntax.take(),
        u_config.syntax.take(),
    ));
    config.cache = Some(override_cache(config.cache.take(), u_config.cache.take()));
    for (key, value) in u_config.headers {
        config.headers.insert(key, value);
    }
}

fn override_format(
    config: Option<FormatSection>,
    u_config: Option<FormatSection>,
) -> FormatSection {
    let mut format = config.unwrap_or_else(|| FormatSection {
        ignored_tags: Vec::new(),
        block_elements: Vec::new(),
        indent_elements: Vec::new(),
    });

    if let Some(u_format) = u_config {
        if !u_format.ignored_tags.is_empty() {
            format.ignored_tags = u_format.ignored_tags;
        }
        if !u_format.block_elements.is_empty() {
            format.block_elements = u_format.block_elements;
        }
        if !u_format.indent_elements.is_empty() {
            format.indent_elements = u_format.indent_elements;
        }
    }
    format
}

fn override_syntax(
    config: Option<SyntaxHighlightingSection>,
    u_config: Option<SyntaxHighlightingSection>,
) -> SyntaxHighlightingSection {
    let mut syntax = config.unwrap_or(SyntaxHighlightingSection {
        theme: None,
        default_language: None,
    });
    if let Some(u_syntax) = u_config {
        if let Some(theme) = u_syntax.theme {
            syntax.theme = Some(theme);
        }
        if let Some(default_language) = u_syntax.default_language {
            syntax.default_language = Some(default_language);
        }
    }
    syntax
}

fn override_cache(config: Option<CacheSection>, u_config: Option<CacheSection>) -> CacheSection {
    let mut cache = config.unwrap_or(CacheSection {
        cache_mode: None,
        max_size: None,
        ttl: None,
    });
    if let Some(u_cache) = u_config {
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
    cache
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_override_defaults() {
        let mut default_config = SiteRawConfig {
            styles: HashMap::new(),
            format: Some(FormatSection {
                ignored_tags: vec!["script".to_string()],
                block_elements: vec!["div".to_string()],
                indent_elements: vec!["li".to_string()],
            }),
            syntax: Some(SyntaxHighlightingSection {
                theme: Some("dark".to_string()),
                default_language: Some("rust".to_string()),
            }),
            cache: None,
            headers: Default::default(),
        };

        let user_config = SiteRawConfig {
            styles: HashMap::new(),
            format: Some(FormatSection {
                ignored_tags: vec!["style".to_string()],
                block_elements: vec![],
                indent_elements: vec!["li".to_string()],
            }),
            syntax: Some(SyntaxHighlightingSection {
                theme: Some("light".to_string()),
                default_language: None,
            }),
            cache: None,
            headers: Default::default(),
        };

        override_defaults_site(&mut default_config, user_config);

        // Format Tests
        assert_eq!(
            default_config.format.as_ref().unwrap().ignored_tags,
            vec!["style"]
        );
        assert_eq!(
            default_config.format.as_ref().unwrap().block_elements,
            vec!["div"]
        );

        assert_eq!(
            default_config.format.as_ref().unwrap().indent_elements,
            vec!["li"]
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
    }

    pub fn override_defaults_site(config: &mut SiteRawConfig, mut u_config: SiteRawConfig) {
        config.format = Some(override_format(
            config.format.take(),
            u_config.format.take(),
        ));
        config.syntax = Some(override_syntax(
            config.syntax.take(),
            u_config.syntax.take(),
        ));
        config.cache = Some(override_cache(config.cache.take(), u_config.cache.take()));
        for (key, value) in u_config.headers {
            config.headers.insert(key, value);
        }
    }
}
