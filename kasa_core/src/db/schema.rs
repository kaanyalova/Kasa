use pest::pratt_parser::Op;
use rayon::str;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, Encode};
use strum::EnumString;

/// Info about Media of all types
#[derive(FromRow, Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Media {
    pub hash: String,
    pub media_type: String,
    pub thumb_path: Option<String>, // can be removed

    // Thumbnails might be generated with different resolutions in the thumbs database compared to,
    // this database, this should not matter as the layouts are calculated using aspect ratio instead,
    // of resolution, though it might make sense to replace this with aspect ratio instead
    pub thumbnail_x: i64,
    pub thumbnail_y: i64,

    pub filesize: i64,
    pub mime: Option<String>,
    pub time_added: i64,

    //pub imported_from: String,
    pub has_file_ref: bool,

    pub hide: bool,
}

// Possible values of `media_type`
#[derive(Debug, Serialize, Deserialize, PartialEq, Encode, EnumString, Clone, Copy, Eq, Hash)]
pub enum MediaType {
    Image,
    Video,
    Game,
    Group,
    Unknown,
}

/// sqlx doesn't serialize them
pub fn media_type_to_string(i: &MediaType) -> String {
    let output = match i {
        MediaType::Image => "Image",
        MediaType::Video => "Video",
        MediaType::Game => "Game",
        MediaType::Unknown => "Unknown",
        MediaType::Group => "Group",
    };
    output.to_string()
}

/// File paths, a file can have multiple of them, files are identified by their hashes
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Path {
    pub hash: String,
    pub path: String,
    pub imported_from: String, // TODO add this
}

/// Basic `Tag` table only used for tag names and FTS searching in tags
#[derive(Serialize, Deserialize, FromRow, Debug, Clone, specta::Type)]
pub struct Tag {
    pub name: String,
}

/// File-tag pairs
#[derive(Serialize, Deserialize, FromRow, Debug)]
pub struct HashTagPair {
    pub hash: String,
    pub tag_name: String,
    pub source: Option<String>,
    /// Tag "category" from the source
    pub source_type: Option<String>,
}

/// Additional information about `Image` media type
#[derive(Debug, FromRow, Clone)]
pub struct Image {
    pub hash: String,
    pub resolution_x: i64,
    pub resolution_y: i64,
}

/// Raw user input of the tags field
#[derive(Debug, FromRow, Clone)]
pub struct RawTagsField {
    #[sqlx(rename = "_text")]
    pub text: String,
}

/// Additional Tag details, all info about tags is here instead of `Tag` table, so we don't deal with limitations
/// of virtual tables
#[derive(Debug, FromRow, Clone, Serialize, Deserialize, specta::Type)]
#[allow(unused)]
pub struct TagDetail {
    name: String,
    /// Should the tag be deleted when there is no `HashTagPair`s containing this tag left
    delete_on_no_references_left: bool,
    color: Option<String>,
    #[sqlx(rename = "_group")]
    group: Option<String>,
    /// Should this tag use its own color instead of the group one
    override_group_color: bool,
}

#[allow(unused)]
pub struct TagGroup {
    name: String,
    color: Option<String>,
}

pub struct MediaGroup {
    group_hash: String,
    group_name: Option<String>,
}

pub struct MediaGroupEntry {
    group_hash: String,
    hash: String,
}

pub struct MediaSource {
    hash: String,
    source: String,
}
