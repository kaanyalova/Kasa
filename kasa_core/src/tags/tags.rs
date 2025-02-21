use std::collections::HashSet;

use itertools::Itertools;
use kasa_python::ExtractedTag;
use log::trace;
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as, query_scalar, Pool, Sqlite};

use crate::media::TagWithDetails;

// Utilities to keep `Tag` and `TagFTS` tables in sync

/// Inserts a tag into the database
///
/// If the media_hash is `Some(_)` it creates the tags into `Tag` table and creates the `HashTagPairs` from provided tags
/// and hash, if it is `None` it only creates the tags in the `Tag` table.
pub async fn insert_tags(
    tags: Vec<String>,
    pool: &Pool<Sqlite>,
    media_hash: Option<String>,
    source: Option<String>,
) {
    // This runs a query for every tag, might be possible to optimize this somehow?

    // Insert tag pairs into HashTagPair
    // Check if tags exist on Tag and TagFTS
    // If it exists on Tag bump the count by one
    let mut tx = pool.begin().await.unwrap();

    if let Some(media_hash) = media_hash {
        for tag in &tags {
            query("INSERT INTO HashTagPair(hash, tag_name, source) VALUES (?, ?, ?) ON CONFLICT DO NOTHING")
                .bind(&media_hash)
                .bind(tag)
                .bind(&source)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    for tag in &tags {
        let does_tag_exist: Option<i64> = query_scalar("SELECT 1 FROM Tag WHERE name = ? ")
            .bind(tag)
            .fetch_optional(pool)
            .await
            .unwrap();

        let does_tag_exist = match does_tag_exist {
            Some(_) => true,
            None => false,
        };

        if !does_tag_exist {
            query("INSERT INTO Tag(name) VALUES (?)")
                .bind(tag)
                .execute(&mut *tx)
                .await
                .unwrap();

            query("INSERT INTO TagDetail(name) VALUES (?)")
                .bind(tag)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    tx.commit().await.unwrap();
}

pub async fn insert_tags_with_source_types(
    tags: Vec<ExtractedTag>,
    pool: &Pool<Sqlite>,
    media_hash: Option<String>,
    source: Option<String>,
) {
    // This runs a query for every tag, might be possible to optimize this somehow?
    trace!("Inserting gallery_dl tags: {:#?}", &tags);
    // Insert tag pairs into HashTagPair
    // Check if tags exist on Tag and TagFTS
    // If it exists on Tag bump the count by one
    let mut tx = pool.begin().await.unwrap();

    if let Some(media_hash) = media_hash {
        for tag in &tags {
            query("INSERT INTO HashTagPair(hash, tag_name, source, source_type) VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING")
                .bind(&media_hash)
                .bind(&tag.name)
                .bind(&source)
                .bind(&tag._type)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    for tag in &tags {
        let does_tag_exist: Option<i64> = query_scalar("SELECT 1 FROM Tag WHERE name = ? ")
            .bind(&tag.name)
            .fetch_optional(pool)
            .await
            .unwrap();

        let does_tag_exist = match does_tag_exist {
            Some(_) => true,
            None => false,
        };

        if !does_tag_exist {
            query("INSERT INTO Tag(name) VALUES (?)")
                .bind(&tag.name)
                .execute(&mut *tx)
                .await
                .unwrap();

            query("INSERT INTO TagDetail(name) VALUES (?)")
                .bind(&tag.name)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    tx.commit().await.unwrap();
}

/// Removes tags from the database
///
/// If the media_hash is Some(_) it removes any `HashTagPairs` along with `Tag`s
/// if it is none it only removes `Tag`s

pub async fn remove_tags(tags: Vec<String>, pool: &Pool<Sqlite>, media_hash: Option<String>) {
    let mut tx = pool.begin().await.unwrap();

    if let Some(media_hash) = media_hash {
        for tag in &tags {
            query("DELETE FROM HashTagPair WHERE hash = ? AND tag_name = ?")
                .bind(&media_hash)
                .bind(tag)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    for tag in tags {
        let tag_reference_count: i64 =
            query_scalar("SELECT COUNT(*) FROM HashTagPair WHERE tag_name = ?")
                .bind(&tag)
                .fetch_one(pool)
                .await
                .unwrap();

        // Delete the row if there is only one reference to the tag, `tag_reference_count` query should execute
        // before this one, so we check if there is one reference left
        if tag_reference_count == 1 {
            // check if the we want to remove the tag if there are no references left
            // the default behavior should remove the tags if no references are left
            // but the user might want to change that on a per tag basis or using a global config option

            let should_cleanup_tags_per_tag = true;

            /*
                query("SELECT should_cleanup_tag FROM TagDetails WHERE tag_name = ?")
                    .bind(tag)
                    .fetch_optional(pool)
                    .await
                    .unwrap(); // TODO Table not implemented

            let should_cleanup_tags_per_tag = match should_cleanup_tags_per_tag {
                Some(_) => true,
                None => false,
            };
            */

            let should_cleanup_tags_global_config = true; // TODO Global config

            if should_cleanup_tags_global_config || should_cleanup_tags_per_tag {
                query("DELETE From Tag WHERE name = ?")
                    .bind(&tag)
                    .execute(&mut *tx)
                    .await
                    .unwrap();
            }
        }
    }

    tx.commit().await.unwrap();
}

pub fn parse_tags(input: &str) -> Vec<&str> {
    input
        .split(",")
        .map(|i| i.trim())
        .filter(|i| i != &"")
        .collect()
}

/// Update the tags in the db according to given `raw_input` which is the list of tags for a given post
/// to be parsed.
///
/// Existing tags are diffed with the newer tags, then tags are added/removed to `HashTagPair` table
/// `HashTagPair` includes the hash of media and the name of the tag.
///
/// Any non existing tags are then added/removed Tags` and `TagsFTS` tables. They should be kept in sync.
///  
///
/// TODOS
/// - Implicit tags
/// - Make sure there to deduplicate the HashTagPairs
pub async fn update_tags_impl(raw_input: &str, hash: String, pool: &Pool<Sqlite>) {
    // TODO RawTagsField removal
    query(
        "INSERT INTO RawTagsField(hash, _text) VALUES (? , ?) ON CONFLICT DO UPDATE SET _text = ?",
    )
    .bind(&hash)
    .bind(raw_input)
    .bind(raw_input)
    .execute(pool)
    .await
    .unwrap();

    let tags = parse_tags(raw_input);

    let previous_tags: Vec<String> =
        query_scalar("SELECT tag_name FROM HashTagPair WHERE hash = ?")
            .bind(&hash)
            .fetch_all(pool)
            .await
            .unwrap();

    let previous_tags_hs: HashSet<&str> =
        HashSet::from_iter(previous_tags.iter().map(|tag| tag.as_str()));
    let tags_hs: HashSet<&str> = HashSet::from_iter(tags);

    let to_add: Vec<String> = tags_hs
        .difference(&previous_tags_hs)
        .cloned()
        .map(|t| t.to_string())
        .collect();

    let to_remove: Vec<String> = previous_tags_hs
        .difference(&tags_hs)
        .cloned()
        .map(|t| t.to_string())
        .collect();

    trace!("---TAG TRANSACTION---");
    trace!("Previous tags: {:?}", previous_tags_hs);
    trace!("Current tags: {:?}", tags_hs);
    trace!("Adding tags to db : {:?}", to_add);
    trace!("Removing tags from db: {:?}", to_remove);
    trace!("---------------------");

    insert_tags(to_add, pool, Some(hash.clone()), None).await;
    remove_tags(to_remove, pool, Some(hash)).await;
}

pub async fn get_tags_as_text_impl(hash: &str, pool: &Pool<Sqlite>) -> String {
    let tags: Vec<String> = query_scalar("SELECT tag_name FROM HashTagPair WHERE hash = ?")
        .bind(&hash)
        .fetch_all(pool)
        .await
        .unwrap();

    tags.iter().join(", ")
}

#[derive(Debug, Serialize, Deserialize, FromRow, specta::Type)]
pub struct TagWithCount {
    pub count: u32,
    pub tag_name: String,
}

pub async fn get_list_of_all_tags_with_details_impl(pool: &Pool<Sqlite>) -> Vec<TagWithCount> {
    query_as("SELECT tag_name, COUNT(tag_name) FROM HashTagPair, TagDetail WHERE HashTagPair.tag_name = TagDetail.name GROUP BY HashTagPair.tag_name")
        .fetch_all(pool)
        .await
        .unwrap()
}
