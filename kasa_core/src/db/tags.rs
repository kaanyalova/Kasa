use std::sync::Arc;

use sqlx::{query, query_scalar, Pool, Sqlite, Transaction};

// Utilities to keep `Tag` and `TagFTS` tables in sync

/// Inserts a tag into the database
///
/// If the media_hash is `Some(_)` it creates the tags into `Tag` table and creates the `HashTagPairs` from provided tags
/// and hash, if it is `None` it only creates the tags in the `Tag` table.
pub async fn insert_tags(tags: Vec<&str>, pool: &Pool<Sqlite>, media_hash: Option<&str>) {
    // This runs a query for every tag, might be possible to optimize this somehow?

    // Insert tag pairs into HashTagPair
    // Check if tags exist on Tag and TagFTS
    // If it exists on Tag bump the count by one
    let mut tx = pool.begin().await.unwrap();

    if let Some(media_hash) = media_hash {
        for tag in &tags {
            query("INSERT INTO HashTagPair(hash, tag_name) VALUES (?, ?)")
                .bind(media_hash)
                .bind(tag)
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

/// Removes tags from the database
///
/// If the media_hash is Some(_) it removes any `HashTagPairs` along with `Tag`s
/// if it is none it only removes `Tag`s

pub async fn remove_tags(tags: Vec<&str>, pool: &Pool<Sqlite>, media_hash: Option<&str>) {
    let mut tx = pool.begin().await.unwrap();

    if let Some(media_hash) = media_hash {
        for tag in &tags {
            query("DELETE FROM HashTagPair WHERE hash = ? AND tag_name = ?")
                .bind(media_hash)
                .bind(tag)
                .execute(&mut *tx)
                .await
                .unwrap();
        }
    }

    for tag in tags {
        let tag_reference_count: i64 =
            query_scalar("SELECT COUNT(*) FROM HashTagPair WHERE tag_name = ?")
                .bind(tag)
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
                    .bind(tag)
                    .execute(&mut *tx)
                    .await
                    .unwrap();
            }
        }
    }

    tx.commit().await.unwrap();
}
