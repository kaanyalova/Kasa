use std::{
    collections::HashMap,
    fs::{self},
};

use log::trace;

use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::ExtractedTag;

pub fn extract_tags(
    extractors: &HashMap<String, ExtractorConfig>,
    gdl_value: &Value,
) -> Result<Vec<ExtractedTag>> {
    let gdl_extractor_name = gdl_value["extractor"].as_str();
    let gdl_extractor_name = match gdl_extractor_name {
        Some(name) => name,
        None => return Err(TagExtractorError::NoExtractorNameFoundOnMedia.into()),
    };

    let extractor = extractors.get(gdl_extractor_name);
    let extractor = match extractor {
        Some(e) => e,
        None => return Ok(vec![]), // No extractor found for metadata, just return with no tags
    };

    let tags = from_toml_extractor(gdl_value, extractor)?;

    Ok(tags)
}

/// Extract the tags from given gallery_dl output given correct extractor toml
///
/// TODO: check if gallery_dl returns empty arrays on missing fields on URLExtractor
fn from_toml_extractor(
    gdl_json: &Value,
    extractor_config: &ExtractorConfig,
) -> Result<Vec<ExtractedTag>> {
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
                    tags.push(ExtractedTag {
                        _type: extractor.category.clone(),
                        name: item.to_owned(),
                    });
                } else {
                    return Err(TagExtractorError::ExtractorArrayDoesNotContainString.into());
                }
            }
        } else if let Some(_str) = json.as_str() {
            // parse the string as a single tag, useful for
            if let Some(false) = extractor.is_split {
                tags.push(ExtractedTag {
                    _type: extractor.category.clone(),
                    name: _str.to_owned(),
                });
            }
            // split the tags using space
            // might want to make this configurable?
            else {
                _str.split(" ").into_iter().for_each(|t| {
                    tags.push(ExtractedTag {
                        _type: extractor.category.clone(),
                        name: t.to_owned(),
                    });
                });
            }
        } else {
            return Err(TagExtractorError::ExtractorPathNotListOrString.into());
        }
    }
    Ok(tags)
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
    tag_extractor: Vec<TagExtractor>,
}

impl ExtractorConfig {
    fn from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let extractor_config: ExtractorConfig = toml::from_str(&contents)?;
        Ok(extractor_config)
    }

    fn from_str(str: &str) -> Result<Self> {
        let extractor_config: ExtractorConfig = toml::from_str(&str)?;
        Ok(extractor_config)
    }
}

pub fn get_extractors_from_path(path: &str) -> Result<HashMap<String, ExtractorConfig>> {
    let extractors = std::fs::read_dir(path)?
        .into_iter()
        .filter_map(|p| p.ok())
        .filter(|f| f.file_type().is_ok())
        .filter(|f| f.file_type().unwrap().is_file())
        .map(|f| f.path())
        .filter_map(|f| ExtractorConfig::from_file(&f.to_string_lossy()).ok())
        .map(|f| (f.extractor_name.clone(), f))
        .collect();

    Ok(extractors)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TagExtractor {
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

    let extractor_config = ExtractorConfig::from_str(&toml).unwrap();

    let tags = from_toml_extractor(&json, &extractor_config).unwrap();

    let expected_tags = vec![
        ExtractedTag::new("test1", "tag1"),
        ExtractedTag::new("test1", "tag2"),
        ExtractedTag::new("test2", "tag3"),
        ExtractedTag::new("test2", "tag4"),
        ExtractedTag::new("spaced", "spaced1"),
        ExtractedTag::new("spaced", "spaced2"),
        ExtractedTag::new("spaced", "spaced3"),
        ExtractedTag::new("not_spaced", "spaced1 spaced2 spaced3"),
    ];

    assert_eq!(tags, expected_tags);
}
