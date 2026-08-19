use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub mod configurable;
pub mod scriptable;

pub trait TagExtractor {
    fn extract_tags(&self, extractor: &str, json_input: &Value) -> Result<ExtractedTags>;
    fn reload(&self) -> Result<()>;
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ExtractedTagWithCategory {
    category: String,
    tag: String,
}

impl ExtractedTagWithCategory {
    pub fn new(category: &str, tag: &str) -> Self {
        Self {
            category: category.to_owned(),
            tag: tag.to_owned(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ExtractedTagWithoutCategory {
    tag: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtractedTags {
    WithCategories(Vec<ExtractedTagWithCategory>),
    WithoutCategories(Vec<ExtractedTagWithoutCategory>),
    Flat(Vec<String>),
    #[serde(skip)]
    NoTags(NoTagsInfo),
}

#[derive(Debug)]
pub struct ExtractedTag {
    pub category: Option<String>,
    pub tag: String,
}

impl ExtractedTags {
    pub fn flatten(self) -> Vec<ExtractedTag> {
        match self {
            ExtractedTags::WithCategories(items) => items
                .into_iter()
                .map(|t| ExtractedTag {
                    category: Some(t.category),
                    tag: t.tag,
                })
                .collect(),
            ExtractedTags::WithoutCategories(items) => items
                .into_iter()
                .map(|t| ExtractedTag {
                    category: None,
                    tag: t.tag,
                })
                .collect(),
            ExtractedTags::Flat(tags) => tags
                .into_iter()
                .map(|tag| ExtractedTag {
                    category: None,
                    tag,
                })
                .collect(),
            ExtractedTags::NoTags(_no_tags_info) => vec![],
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum NoTagsInfo {
    NoExtractorFound,
    NoTags,
}
