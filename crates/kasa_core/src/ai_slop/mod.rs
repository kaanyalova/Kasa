use std::{cmp::Ordering, hash::Hash};

use serde::{Deserialize, Serialize};

pub mod a1111;
pub mod comfy;
pub mod errors;
mod prompt_parser;
pub mod supported_formats;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SlopTags {
    pub positive: Vec<SlopTag>,
    pub negative: Vec<SlopTag>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
/// Only hashed by `name`
pub struct SlopTag {
    pub name: String,
    pub power: f64,
}

impl Eq for SlopTag {}
impl Ord for SlopTag {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.as_str().cmp(other.name.as_str())
    }
}

impl PartialOrd for SlopTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for SlopTag {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        //self.power.hash(state);
    }
}

#[allow(unused)]
impl SlopTag {
    fn new(name: &str, power: f64) -> Self {
        SlopTag {
            name: name.to_string(),
            power,
        }
    }
}
