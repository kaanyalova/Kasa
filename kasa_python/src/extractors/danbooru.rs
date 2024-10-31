use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ExtractedTag;

#[derive(Serialize, Deserialize, Debug)]
pub struct Danbooru {
    //approver_id: Option<i64>,
    //bit_flags: i64,
    tags_general: Vec<String>,
    tags_artist: Vec<String>,
    tags_character: Vec<String>,
    tags_copyright: Vec<String>,
    tags_meta: Vec<String>,

    // other values that are either not in this struct or newly added
    #[serde(flatten)]
    others: Value,
}

impl Danbooru {
    pub fn tags(&self) -> Vec<ExtractedTag> {
        let mut tags: Vec<ExtractedTag> = vec![];

        tags.extend(self.tags_artist.iter().map(|t| ExtractedTag {
            _type: "Artist".to_string(),
            name: t.to_owned(),
        }));

        tags.extend(self.tags_character.iter().map(|t| ExtractedTag {
            _type: "Characters".to_string(),
            name: t.to_owned(),
        }));

        tags.extend(self.tags_copyright.iter().map(|t| ExtractedTag {
            _type: "Copyright".to_string(),
            name: t.to_owned(),
        }));

        tags.extend(self.tags_general.iter().map(|t| ExtractedTag {
            _type: "General".to_string(),
            name: t.to_owned(),
        }));

        tags.extend(self.tags_meta.iter().map(|t| ExtractedTag {
            _type: "Meta".to_string(),
            name: t.to_owned(),
        }));

        tags
    }
}
