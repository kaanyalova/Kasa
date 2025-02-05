use std::{collections::HashMap, path::PathBuf, str::FromStr};

use chrono::{DateTime, Local, TimeZone, Utc};
use ffmpeg::filter::Source;
use human_bytes::human_bytes;
use itertools::Itertools;
use rustpython_vm::common::str;
use serde::{Deserialize, Serialize};
use sqlx::{query_as, query_scalar, Pool, Sqlite};

use crate::db::schema::{HashTagPair, Image, Media, MediaType, RawTagsField};

/// Gets all the info to show to user in the sidebar for a piece of media
pub async fn get_info_impl(hash: &str, pool: &Pool<Sqlite>) -> MediaInfo {
    let media: Media = query_as("SELECT * FROM Media WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap();

    let paths: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_all(pool)
        .await
        .unwrap();

    let _type = MediaType::from_str(&media.media_type).unwrap();

    let mut meta: Vec<MetaEntry> = vec![];

    // meta entries for all formats

    if media.mime.is_some() {
        meta.push(MetaEntry {
            name: "File Type".to_string(),
            value: media.mime.as_ref().unwrap().clone(),
            is_value_monospaced: true,
            is_one_line: true,
        });
    }

    meta.push(MetaEntry {
        name: "File Size".to_string(),
        value: human_bytes(media.filesize as f64),
        is_value_monospaced: true,
        is_one_line: true,
    });

    let datetime = Utc.timestamp_millis_opt(media.time_added).unwrap();
    let local_datetime: DateTime<Local> = DateTime::from(datetime);
    let human_readable_time = local_datetime.format("%d %b %y %X").to_string();

    meta.push(MetaEntry {
        name: "Time Added".to_string(),
        value: human_readable_time,
        is_value_monospaced: false,
        is_one_line: true,
    });

    meta.push(MetaEntry {
        name: "Hash".to_string(),
        value: media.hash.clone(),
        is_value_monospaced: true,
        is_one_line: true,
    });

    // Meta entries for specific formats
    match _type {
        MediaType::Image => {
            let q: Image = query_as("SELECT * FROM Image WHERE hash = ?")
                .bind(hash)
                .fetch_one(pool)
                .await
                .unwrap();

            let resolution = format!("{} x {}", q.resolution_x, q.resolution_y);

            meta.push(MetaEntry {
                name: "Resolution".to_string(),
                value: resolution,
                is_value_monospaced: true,
                is_one_line: true,
            })
        }
        MediaType::Video => { /* TODO implement video meta */ }
        MediaType::Game => unimplemented!(),
        MediaType::Unknown => unimplemented!(),
        MediaType::Group => unimplemented!(),
        MediaType::Flash => { /* TODO implement flash meta */ }
    };

    let import = ImportInfo {
        import_source: "Placeholder".to_string(),
        import_link: Some("https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string()),
    };

    let tags = get_tags_impl(hash, pool).await;

    // The user might write invalid syntax to the Tags input box in that case we don't want to remove the user input
    // and replace it with the HashTagPair entries, we probably want to warn the user in the UI though
    //let raw_tags_field_from_db: Option<RawTagsField> =
    //    query_as("SELECT * FROM RawTagsField WHERE hash = ? ")
    //        .bind(&media.hash)
    //        .fetch_optional(pool)
    //        .await
    //        .unwrap();

    // TODO RawTagsField removal: Remove RawTagsField stuff, text input might be a thing but storing the original input is too much of
    // an hassle when tags get added/removed using the gui

    let raw_tags_field_from_db: Option<RawTagsField> = None;

    let tags_len = tags.len();

    let raw_tags_field: String = match raw_tags_field_from_db {
        Some(tags) => tags.text,
        None => {
            // If the image has auto generated tags we want to generate the raw_tags_field on demand
            if tags_len > 0 {
                tags.iter().map(|t| t.name.clone()).join(", ")
            } else {
                "".to_string()
            }
        }
    };

    let aspect_ratio = media.thumbnail_x as f64 / media.thumbnail_y as f64;

    let file_name = PathBuf::from(&paths[0])
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    // Group the tags according to their `source_category`es

    let mut tags_with_no_source_types = vec![];
    let mut tags_with_source_types: HashMap<String, Vec<MediaTag>> = HashMap::new();

    tags.iter().for_each(|t| {
        if let Some(category) = &t.source_category {
            //tags_with_source_types.insert(category, t.name.clone());
            let tag_vec = tags_with_source_types.get_mut(category);

            if let Some(tag_vec) = tag_vec {
                tag_vec.push(t.clone());
            } else {
                tags_with_source_types.insert(category.clone(), vec![t.clone()]);
            }
        } else {
            tags_with_no_source_types.push(t.clone());
        }
    });

    let source_grouped_types = SourceGroupedTags {
        source_categories: tags_with_source_types,
        uncategorized: tags_with_no_source_types,
    };

    MediaInfo {
        tags,
        meta,
        import,
        paths,
        raw_tags_field,
        hash: hash.to_string(),
        media_type: media.media_type.to_string(),
        mime: {
            // Workaround, mime_guess parses all matroska files as x-matroska
            // we assume they are all video

            if let Some(mime) = media.mime {
                if mime == "video/x-matroska" {
                    Some("video/matroska".to_string())
                } else {
                    Some(mime)
                }
            } else {
                None
            }
        },
        aspect_ratio,
        file_name,
        tags_with_source_types: source_grouped_types,
    }
}

// Gets tag info about a piece of media
pub async fn get_tags_impl(hash: &str, pool: &Pool<Sqlite>) -> Vec<MediaTag> {
    let hash_tag_pairs: Vec<HashTagPair> = query_as("SELECT * FROM HashTagPair WHERE hash = ?")
        .bind(hash)
        .fetch_all(pool)
        .await
        .unwrap();

    // Might need extra info about hashes, this is why we map the has here
    let tags = hash_tag_pairs
        .into_iter()
        .map(|tag| MediaTag {
            name: tag.tag_name,
            source_category: tag.source_type,
        })
        .collect();

    tags
}

pub async fn get_media_type_impl(hash: &str, pool: &Pool<Sqlite>) -> String {
    query_scalar("SELECT media_type FROM Media WHERE hash = ?")
        .bind(hash)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MediaInfo {
    pub meta: Vec<MetaEntry>,
    pub import: ImportInfo,
    pub paths: Vec<String>,
    pub tags: Vec<MediaTag>,
    pub tags_with_source_types: SourceGroupedTags,
    pub raw_tags_field: String,
    pub hash: String,
    pub media_type: String,
    pub mime: Option<String>,
    pub aspect_ratio: f64,
    pub file_name: String,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MetaEntry {
    pub name: String,
    pub value: String,
    pub is_value_monospaced: bool,
    pub is_one_line: bool,
}

#[derive(Debug, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ImportInfo {
    pub import_source: String,
    pub import_link: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MediaTag {
    name: String,
    source_category: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, specta::Type, Clone)]
pub struct SourceGroupedTags {
    source_categories: HashMap<String, Vec<MediaTag>>,
    uncategorized: Vec<MediaTag>,
}
