pub mod db_info;
pub mod migrations;
pub mod schema;
pub mod tags;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, prelude::FromRow, query_as};

use crate::index::media_types::FirstPass;

use schema::{Media, TagDetail};

#[derive(Serialize, Deserialize, FromRow, Debug, Clone, specta::Type)]
pub struct TagQueryOutput {
    name: String,
    #[sqlx(rename = "tag_count")]
    count: i64,

    #[sqlx(flatten)]
    tag_details: TagDetail,
}

/// Queries for the first n tags using `tag*`, if there is less than n it queries `*tag*` for (limit - n) tags
/// and adds that to result
pub async fn query_tags_impl(
    _query: String,
    limit: i64,
    pool: &Pool<Sqlite>,
) -> Vec<TagQueryOutput> {
    let wildcard = format!("{}%", _query);

    let mut tags = vec![];

    let mut q: Vec<TagQueryOutput> =
    // https://stackoverflow.com/questions/4784545/sql-how-to-order-using-count-from-another-table
    // SELECT * FROM Tag WHERE name LIKE ? ORDER BY count DESC LIMIT 
    // SELECT Tag.* , COUNT(tag) AS tag_count , TagDetail.* FROM Tag LEFT JOIN HashTagPair ON HashTagPair.tag_name = Tag.name LEFT JOIN TagDetail ON TagDetail.name = Tag.name WHERE tag_name LIKE "x" GROUP BY Tag.name ORDER BY tag_count DESC LIMIT 5;
        query_as("SELECT Tag.* , COUNT(tag) AS tag_count , TagDetail.* FROM Tag LEFT JOIN HashTagPair ON HashTagPair.tag_name = Tag.name LEFT JOIN TagDetail ON TagDetail.name = Tag.name WHERE tag_name LIKE ? GROUP BY Tag.name ORDER BY tag_count DESC LIMIT ?")
            .bind(wildcard)
            .bind(limit)
            .fetch_all(pool)
            .await
            .unwrap();

    tags.append(&mut q);

    if (tags.len() as i64) < limit {
        let new_limit = limit - (tags.len() as i64);
        let wildcard = format!("%{}%", _query);
        let mut q: Vec<TagQueryOutput> =
            query_as("SELECT Tag.* , COUNT(tag) AS tag_count , TagDetail.* FROM Tag LEFT JOIN HashTagPair ON HashTagPair.tag_name = Tag.name LEFT JOIN TagDetail ON TagDetail.name = Tag.name WHERE tag_name LIKE ? GROUP BY Tag.name ORDER BY tag_count DESC LIMIT ?")
                .bind(wildcard)
                .bind(new_limit)
                .fetch_all(pool)
                .await
                .unwrap();

        tags.append(&mut q);
    }

    tags.iter()
        .unique_by(|tag| &tag.name)
        .cloned()
        .sorted_by_key(|t| t.count)
        .rev()
        .collect()
}

pub async fn query_all_test_impl(
    //_query: String,
    pagination: i64,
    page: i64,
    pool: &Pool<Sqlite>,
) -> Vec<FirstPass> {
    let offset = pagination * page;

    let q: Vec<FirstPass> = query_as("SELECT * FROM TestMedia LIMIT ? OFFSET ?")
        .bind(pagination)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap();

    q
}

pub async fn query_all_test_impl2(
    //_query: String,
    pagination: i64,
    page: i64,
    pool: &Pool<Sqlite>,
) -> Vec<FirstPass> {
    let offset = pagination * page;

    let q: Vec<FirstPass> = query_as("SELECT * FROM TestMedia LIMIT ? OFFSET ?")
        .bind(pagination)
        .bind(offset)
        .fetch_all(pool)
        .await
        .unwrap();

    q
}

pub async fn get_all_media_impl(pool: &Pool<Sqlite>) -> Vec<Media> {
    query_as("SELECT * FROM Media")
        .fetch_all(pool)
        .await
        .unwrap()
}
