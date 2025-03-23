use crate::config::color_conversion::Style;
use scraper::ElementRef;
use std::collections::{BTreeMap, HashMap, HashSet};

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub enum TagData {
    #[default]
    None,
    Styled(Style),
}

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub struct TagDataIdentifier {
    data: TagData,
    classes: BTreeMap<String, TagData>,
    ids: BTreeMap<String, TagData>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
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
    pub tag_styles: HashMap<String, TagDataIdentifier>,
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
        let tag_styles_map = Self::build_data_map_from_selectors(
            tag_styles
                .into_iter()
                .map(|(str, sty)| (str, TagData::Styled(sty)))
                .collect(),
        );
        Self {
            ignored_tags: ignored_tags_map,
            block_elements: block_elements_map,
            indent_elements: indent_elements_map,
            tag_styles: tag_styles_map,
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

    fn build_data_map_from_selectors(
        tag_to_data: HashSet<(String, TagData)>,
    ) -> HashMap<String, TagDataIdentifier> {
        let mut ignored_tags_map: HashMap<String, TagDataIdentifier> = HashMap::new();
        for (tag, tag_data) in tag_to_data {
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
                tag_identifier.data = tag_data.clone();
            }
            for class in classes {
                tag_identifier
                    .classes
                    .insert(class.to_string(), tag_data.clone());
            }
            if let Some(id) = id {
                tag_identifier.ids.insert(id.to_string(), tag_data.clone());
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

    pub fn style_for_tag(&self, element: &ElementRef) -> Option<Style> {
        let general_identifier = self.tag_styles.get("");
        let mut head_style = vec![];
        let mut class_style = vec![];
        let mut id_style = vec![];
        if let Some(general) = general_identifier {
            Self::insert_values(
                element,
                general,
                &mut head_style,
                &mut class_style,
                &mut id_style,
            );
        }
        let element_tag = element.value().name();
        let tag_identifier = self.tag_styles.get(element_tag);
        if let Some(tag) = tag_identifier {
            Self::insert_values(
                element,
                tag,
                &mut head_style,
                &mut class_style,
                &mut id_style,
            );
        }
        let mut all_styles = vec![];
        all_styles.extend(head_style);
        all_styles.extend(class_style);
        all_styles.extend(id_style);
        if all_styles.is_empty() {
            return None;
        }
        let mut merged_style = Style::default();
        for style in all_styles {
            merged_style = merged_style.patch(&style);
        }
        Some(merged_style)
    }

    fn insert_values(
        element: &ElementRef,
        identifier: &TagDataIdentifier,
        head_style: &mut Vec<Style>,
        class_style: &mut Vec<Style>,
        id_style: &mut Vec<Style>,
    ) {
        if let TagData::Styled(style) = identifier.data.clone() {
            head_style.push(style);
        }
        if let Some(classes) = element.value().attr("class") {
            class_style.extend(classes.split_whitespace().filter_map(|class| {
                identifier.classes.get(class).and_then(|td| match td {
                    TagData::Styled(style) => Some(style),
                    TagData::None => None,
                })
            }));
        }
        if let Some(id) = element.value().id() {
            if let Some(TagData::Styled(style)) = identifier.ids.get(id) {
                id_style.push(*style);
            }
        }
    }
}
