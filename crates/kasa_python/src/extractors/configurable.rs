use std::{
    collections::HashMap,
    fs::{self},
    path::Path,
    str::FromStr,
};

use log::trace;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::extractors::{ExtractedTagWithCategory, ExtractedTags, TagExtractor};

pub struct ConfigurableExtractor {
    extractors: HashMap<String, ExtractorConfig>,
}

impl ConfigurableExtractor {
    /// Extract the tags from given gallery_dl output given correct extractor toml
    ///
    /// TODO: check if gallery_dl returns empty arrays on missing fields on URLExtractor
    fn extract_tags_with_config(
        gdl_json: &Value,
        extractor_config: &ExtractorConfig,
    ) -> Result<ExtractedTags> {
        //let extractor_config: ExtractorConfig = toml::from_str(toml)?;
        //let gdl_json: Value = serde_json::from_str(&gdl_json)?;
        let mut tags = vec![];

        trace!("Toml extractor Received gdl_json: {:#?}", &gdl_json);
        trace!("Toml extractor extractor_config: {:#?}", &extractor_config);

        for extractor in &extractor_config.tag_extractor {
            let mut json = gdl_json;
            for key in &extractor.keys {
                match key {
                    Key::Index(i) => {
                        let _json = json.get(i.to_owned() as usize);
                        let _json = match _json {
                            Some(json) => json,
                            None => return Err(TagExtractorError::WrongExtractorPath.into()),
                        };
                        json = _json;
                    }
                    Key::String(str) => {
                        let _json = json.get(str);
                        let _json = match _json {
                            Some(json) => json,
                            None => return Err(TagExtractorError::WrongExtractorPath.into()),
                        };
                        json = _json;
                    }
                };
            }

            if let Some(array) = json.as_array() {
                for item in array {
                    if let Some(item) = item.as_str() {
                        tags.push(ExtractedTagWithCategory {
                            category: extractor.category.clone(),
                            tag: item.to_owned(),
                        });
                    } else {
                        return Err(TagExtractorError::ExtractorArrayDoesNotContainString.into());
                    }
                }
            } else if let Some(_str) = json.as_str() {
                // parse the string as a single tag, useful for
                if let Some(false) = extractor.is_split {
                    tags.push(ExtractedTagWithCategory {
                        category: extractor.category.clone(),
                        tag: _str.to_owned(),
                    });
                }
                // split the tags using space
                // might want to make this configurable?
                else {
                    _str.split(" ").for_each(|t| {
                        tags.push(ExtractedTagWithCategory {
                            category: extractor.category.clone(),
                            tag: t.to_owned(),
                        });
                    });
                }
            } else {
                return Err(TagExtractorError::ExtractorPathNotListOrString.into());
            }
        }
        Ok(ExtractedTags::WithCategories(tags))
    }

    pub fn get_extractors_from_path(path: &Path) -> Result<HashMap<String, ExtractorConfig>> {
        let extractors = std::fs::read_dir(path)?
            .filter_map(|p| p.ok())
            .filter(|f| f.file_type().is_ok())
            .filter(|f| f.file_type().unwrap().is_file())
            .map(|f| f.path())
            .filter_map(|f| ExtractorConfig::from_file(&f.to_string_lossy()).ok())
            .map(|f| (f.extractor_name.clone(), f))
            .collect();

        Ok(extractors)
    }

    pub fn init(extractors_path: &Path) -> Result<Self> {
        let extractors = Self::get_extractors_from_path(extractors_path)?;

        Ok(Self { extractors })
    }
}

impl TagExtractor for ConfigurableExtractor {
    fn extract_tags(&self, extractor: &str, json_input: &Value) -> Result<ExtractedTags> {
        let extractor = self.extractors.get(extractor);
        let extractor_config = match extractor {
            Some(e) => e,
            None => return Ok(ExtractedTags::NoTags(super::NoTagsInfo::NoExtractorFound)),
        };

        Self::extract_tags_with_config(json_input, extractor_config)
    }
}

#[derive(Error, Debug)]
enum TagExtractorError {
    #[error("Extractor provided invalid json path")]
    WrongExtractorPath,
    #[error("Extractor found a data type that was not a string or an array")]
    ExtractorPathNotListOrString,
    #[error("Extractor found an array that does not contain strings")]
    ExtractorArrayDoesNotContainString,
    #[error("The image metadata received from gallery_dl doesn't contain the extractor field")]
    NoExtractorNameFoundOnMedia,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractorConfig {
    extractor_name: String,
    tag_extractor: Vec<TagExtractorConfig>,
}

impl ExtractorConfig {
    fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let extractor_config: ExtractorConfig = toml::from_str(&contents)?;
        Ok(extractor_config)
    }
}
impl FromStr for ExtractorConfig {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let extractor_config: ExtractorConfig = toml::from_str(s)?;
        Ok(extractor_config)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TagExtractorConfig {
    keys: Vec<Key>,
    category: String,
    is_split: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
enum Key {
    Index(u32),
    String(String),
}

#[test]
fn test_config() {
    use serde_json::json;
    let json = json!({
            "one": {
            "two": ["tag1", "tag2"],
            "three": {
                "four": ["tag3", "tag4"]
            },
        },
        "separated_by_space": "spaced1 spaced2 spaced3"
    });

    let toml = r#"
    extractor_name="test"

    [[tag_extractor]]
    keys = ["one", "two"]
    category = "test1"


    [[tag_extractor]]
    keys = ["one", "three", "four"]
    category = "test2"
    
    [[tag_extractor]]
    keys = ["separated_by_space"]
    category = "spaced"
    

    [[tag_extractor]]
    keys = ["separated_by_space"]
    category = "not_spaced"
    is_split = false
    "#;

    let extractor_config = ExtractorConfig::from_str(toml).unwrap();

    let tags = ConfigurableExtractor::extract_tags_with_config(&json, &extractor_config).unwrap();

    let expected_tags = vec![
        ExtractedTagWithCategory::new("test1", "tag1"),
        ExtractedTagWithCategory::new("test1", "tag2"),
        ExtractedTagWithCategory::new("test2", "tag3"),
        ExtractedTagWithCategory::new("test2", "tag4"),
        ExtractedTagWithCategory::new("spaced", "spaced1"),
        ExtractedTagWithCategory::new("spaced", "spaced2"),
        ExtractedTagWithCategory::new("spaced", "spaced3"),
        ExtractedTagWithCategory::new("not_spaced", "spaced1 spaced2 spaced3"),
    ];

    let expected_tags = ExtractedTags::WithCategories(expected_tags);

    assert_eq!(tags, expected_tags);
}
