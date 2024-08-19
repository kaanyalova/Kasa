use std::{collections::HashSet, hash::Hash};

use itertools::Itertools;
use log::trace;
use regex::Regex;
use sqlx::{query, query_as, query_builder, query_scalar, Pool, QueryBuilder, Sqlite};

use crate::db::{
    schema::HashTagPair,
    tags::{insert_tags, remove_tags},
};

pub async fn create_tag() {}
pub async fn remove_tag() {}
pub async fn tag_media_single() {}
pub async fn tag_media_batch() {}

fn parse_tags(input: &str) -> Vec<&str> {
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
pub async fn update_tags_impl(raw_input: &str, hash: &str, pool: &Pool<Sqlite>) {
    query(
        "INSERT INTO RawTagsField(hash, _text) VALUES (? , ?) ON CONFLICT DO UPDATE SET _text = ?",
    )
    .bind(hash)
    .bind(raw_input)
    .bind(raw_input)
    .execute(pool)
    .await
    .unwrap();

    let tags = parse_tags(raw_input);

    let previous_tags: Vec<String> =
        query_scalar("SELECT tag_name FROM HashTagPair WHERE hash = ?")
            .bind(hash)
            .fetch_all(pool)
            .await
            .unwrap();

    let previous_tags_hs: HashSet<&str> =
        HashSet::from_iter(previous_tags.iter().map(|tag| tag.as_str()));
    let tags_hs: HashSet<&str> = HashSet::from_iter(tags);

    let to_add: Vec<&str> = tags_hs.difference(&previous_tags_hs).cloned().collect();
    let to_remove: Vec<&str> = previous_tags_hs.difference(&tags_hs).cloned().collect();

    trace!("---TAG TRANSACTION---");
    trace!("Previous tags: {:?}", previous_tags_hs);
    trace!("Current tags: {:?}", tags_hs);
    trace!("Adding tags to db : {:?}", to_add);
    trace!("Removing tags from db: {:?}", to_remove);
    trace!("---------------------");

    insert_tags(to_add, pool, Some(hash)).await;
    remove_tags(to_remove, pool, Some(hash)).await;
}
