use std::collections::HashMap;

use sqlx::{query_as, Pool, QueryBuilder, Sqlite};

#[allow(unused)]
use crate::{
    db::schema::Media,
    test_util::db_utils::{_insert_media_row, insert_hash_tag_pair_row},
};

use super::tags::parse_tags;

pub fn parse() {
    todo!()
}

// Tags separated by commas
// `tag1, tag2`
//
// OR queries
// `tag1 or tag2, tag3, tag4`
//
// NOT queries,
// `tag1, tag2, not tag3` or `tag1, tag2, -tag3`
//
// ORDERING
// `tag1, tag2, order by reverse date`
//
// TIME queries
// `tag1, imported between 3 days ago and now`
// `tag1, imported yesterday`

#[allow(unused)]
pub fn search(input: Vec<SearchInput>, pool: &Pool<Sqlite>) {
    // HashMap of tag and vector of hashes
    let tags: HashMap<String, Vec<String>> = HashMap::new();
}

pub enum SearchInput {
    Tag(String),
    HaveTag(Box<SearchInput>),
    ExcludeTag(Box<SearchInput>),
    OrTags(Vec<SearchInput>),
    Order(OrderType),
    TimeQueryBetween(u64, u64),
    TimeQueryUntilNow(u64),
}

pub enum OrderType {
    Date,
}

/// Placeholder search until I implement proper search parsing
/// Only supports searching for Media that have the tags
pub async fn search_simple_impl(raw_input: &str, pool: &Pool<Sqlite>) -> Vec<Media> {
    let tags = parse_tags(raw_input);

    // show all Media on empty search
    if tags.len() == 0 {
        return query_as("SELECT * FROM Media")
            .fetch_all(pool)
            .await
            .unwrap();
    }

    let mut query_builder: QueryBuilder<Sqlite> =
        QueryBuilder::new("SELECT m.* FROM HashTagPair htp, Media m WHERE (htp.tag_name) IN (");

    let mut separated = query_builder.separated(", ");

    for tag in &tags {
        separated.push_bind(tag);
    }

    separated.push_unseparated(") ");

    query_builder.push("AND m.hash = htp.hash GROUP BY m.hash HAVING COUNT (m.hash) = ");

    query_builder.push_bind(tags.len() as i32);

    query_builder.push("AND m.has_file_ref = true");

    let query = query_builder.build_query_as::<Media>();

    query.fetch_all(pool).await.unwrap()
}

#[sqlx::test]
fn test_simple_search(pool: Pool<Sqlite>) {
    // set up db
    sqlx::migrate!("../migrations/db").run(&pool).await.unwrap();

    let var_name = Media {
        hash: "123".to_string(),
        media_type: "Image".to_string(),
        thumb_path: Some("nowhere".to_string()),
        thumbnail_x: 123,
        thumbnail_y: 123,
        filesize: 9999,
        mime: None,
        time_added: 0,
        has_file_ref: true,
        hide: false,
    };
    let media = var_name;

    _insert_media_row(&pool, &media).await;

    insert_hash_tag_pair_row("123", "never", &pool).await;
    insert_hash_tag_pair_row("123", "gonna", &pool).await;
    insert_hash_tag_pair_row("123", "give", &pool).await;
    insert_hash_tag_pair_row("123", "you", &pool).await;
    insert_hash_tag_pair_row("123", "up", &pool).await;

    let media_from_db = search_simple_impl("never,gonna,give,you,up", &pool).await;

    assert_eq!(media_from_db.len(), 1);

    assert_eq!(media_from_db[0], media);
}
