use itertools::Itertools;
use sqlx::{query_as, Pool, Sqlite};

use crate::index::media_types::FirstPass;

use super::schema::Tag;

/// Queries for the first n tags using `tag*`, if there is less than n it queries `*tag*` for (limit - n) tags
/// and adds that to result
pub async fn query_tags_impl(_query: String, limit: i64, pool: &Pool<Sqlite>) -> Vec<Tag> {
    let wildcard = format!("{}%", _query);

    let mut tags = vec![];

    let mut q: Vec<Tag> =
        query_as("SELECT * FROM Tag WHERE name LIKE ? ORDER BY count DESC LIMIT ?")
            .bind(wildcard)
            .bind(limit)
            .fetch_all(pool)
            .await
            .unwrap();

    tags.append(&mut q);

    if (tags.len() as i64) < limit {
        let new_limit = limit - (tags.len() as i64);
        let wildcard = format!("%{}%", _query);
        let mut q: Vec<Tag> =
            query_as("SELECT * FROM Tag WHERE name LIKE ? ORDER BY count DESC LIMIT ?")
                .bind(wildcard)
                .bind(new_limit)
                .fetch_all(pool)
                .await
                .unwrap();

        tags.append(&mut q);
    }

    let end_tags = tags.iter().unique_by(|tag| &tag.name).cloned().collect();

    end_tags
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
