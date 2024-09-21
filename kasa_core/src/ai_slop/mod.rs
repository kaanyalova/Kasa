use std::hash::Hash;

use serde::{Deserialize, Serialize};

mod a1111;
mod comfy;
mod errors;
pub mod indexers;
mod prompt_parser;
mod supported_formats;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SlopTags {
    pub positive: Vec<SlopTag>,
    pub negative: Vec<SlopTag>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, PartialOrd)]
/// Only hashed by `name`
pub struct SlopTag {
    pub name: String,
    pub power: f64,
}

impl Eq for SlopTag {}

impl Hash for SlopTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        //self.power.hash(state);
    }
}

impl SlopTag {
    fn new(name: &str, power: f64) -> Self {
        SlopTag {
            name: name.to_string(),
            power,
        }
    }
}
