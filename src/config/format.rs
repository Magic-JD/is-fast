use ratatui::prelude::Style;
use scraper::ElementRef;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default)]
pub struct TagIdentifier {
    unconditional: bool,
    classes: HashSet<String>,
    ids: HashSet<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FormatConfig {
    pub ignored_tags: HashMap<String, TagIdentifier>,
    pub block_elements: HashMap<String, TagIdentifier>,
    pub indent_elements: HashMap<String, TagIdentifier>,
    pub tag_styles: HashMap<String, Style>,
}

impl FormatConfig {
    pub fn new(
        ignored_tags: HashSet<String>,
        block_elements: HashSet<String>,
        indent_elements: HashSet<String>,
        tag_styles: HashMap<String, Style>,
    ) -> Self {
        let ignored_tags_map = Self::build_map_from_selectors(ignored_tags);
        let block_elements_map = Self::build_map_from_selectors(block_elements);
        let indent_elements_map = Self::build_map_from_selectors(indent_elements);
        Self {
            ignored_tags: ignored_tags_map,
            block_elements: block_elements_map,
            indent_elements: indent_elements_map,
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

    pub fn is_indent_element(&self, element: &ElementRef) -> bool {
        let tag_identifier = self.indent_elements.get(element.value().name());
        let general_identifier = self.indent_elements.get("");
        Self::matches_tag(element, tag_identifier) || Self::matches_tag(element, general_identifier)
    }

    pub fn style_for_tag(&self, tag: &str) -> Option<&Style> {
        self.tag_styles.get(tag)
    }
}
