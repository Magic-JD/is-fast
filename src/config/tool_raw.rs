use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
pub struct SearchSection {
    #[serde(default)]
    pub(crate) engine: Option<String>,
    #[serde(default)]
    pub(crate) site: Option<String>,
    #[serde(default)]
    pub(crate) timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MiscSection {
    #[serde(default)]
    pub(crate) open_tool: Option<String>,
    #[serde(default)]
    pub(crate) text_size_supported: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ToolRawConfig {
    #[serde(default)]
    pub(crate) selectors: HashMap<String, String>,
    #[serde(default)]
    pub(crate) display: Option<DisplaySection>,
    #[serde(default)]
    pub(crate) history: Option<HistorySection>,
    #[serde(default)]
    pub(crate) search: Option<SearchSection>,
    #[serde(default)]
    pub(crate) misc: Option<MiscSection>,
    #[serde(default)]
    pub(crate) custom_config: HashMap<String, Vec<String>>,
}

impl ToolRawConfig {
    pub fn default() -> Self {
        Self {
            selectors: HashMap::new(),
            display: None,
            history: None,
            search: None,
            misc: None,
            custom_config: HashMap::new(),
        }
    }
}

pub fn override_defaults_tool(config: &mut ToolRawConfig, mut u_config: ToolRawConfig) {
    for (site, selector) in u_config.selectors {
        config.selectors.insert(site, selector);
    }
    config.display = Some(override_display(
        config.display.take(),
        u_config.display.take(),
    ));
    config.history = Some(override_history(
        config.history.take(),
        u_config.history.take(),
    ));
    config.search = Some(override_search(
        config.search.take(),
        u_config.search.take(),
    ));
    config.misc = Some(override_misc(config.misc.take(), u_config.misc.take()));
    for (site, file) in u_config.custom_config {
        config.custom_config.insert(site, file);
    }
}

fn override_display(
    config: Option<DisplaySection>,
    u_config: Option<DisplaySection>,
) -> DisplaySection {
    let mut display = config.unwrap_or(DisplaySection {
        border_color: None,
        page_margin: None,
        scroll: None,
        color_mode: None,
    });
    if let Some(u_display) = u_config {
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
    display
}

fn override_history(
    config: Option<HistorySection>,
    u_config: Option<HistorySection>,
) -> HistorySection {
    let mut history = config.unwrap_or(HistorySection {
        title_color: None,
        url_color: None,
        time_color: None,
        text_color: None,
        search_type: None,
        enabled: None,
    });

    if let Some(u_history) = u_config {
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
    history
}

fn override_search(
    config: Option<SearchSection>,
    u_config: Option<SearchSection>,
) -> SearchSection {
    let mut search = config.unwrap_or(SearchSection {
        engine: None,
        site: None,
        timeout: 0,
    });

    if let Some(u_search) = u_config {
        if let Some(engine) = u_search.engine {
            search.engine = Some(engine);
        }
        if let Some(site) = u_search.site {
            search.site = Some(site);
        }
        if u_search.timeout > 0 {
            search.timeout = u_search.timeout;
        }
    }
    search
}

fn override_misc(misc: Option<MiscSection>, u_misc: Option<MiscSection>) -> MiscSection {
    let mut misc = misc.unwrap_or(MiscSection {
        open_tool: None,
        text_size_supported: false,
    });

    if let Some(u_misc) = u_misc {
        if let Some(open_tool) = u_misc.open_tool {
            misc.open_tool = Some(open_tool);
        }
        misc.text_size_supported = u_misc.text_size_supported;
    }
    misc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::tool_raw::ToolRawConfig;

    #[test]
    fn test_override_defaults() {
        let mut default_config = ToolRawConfig {
            selectors: {
                let mut map = HashMap::new();
                map.insert("example.com".to_string(), "body".to_string());
                map
            },
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
            misc: None,
            custom_config: Default::default(),
        };

        let user_config = ToolRawConfig {
            selectors: {
                let mut map = HashMap::new();
                map.insert("newsite.com".to_string(), "header".to_string());
                map
            },
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
            misc: None,
            custom_config: Default::default(),
        };

        override_defaults_tool(&mut default_config, user_config);

        // Selector Tests
        assert_eq!(default_config.selectors.len(), 2);
        assert!(default_config.selectors.contains_key("example.com"));
        assert!(default_config.selectors.contains_key("newsite.com"));

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
