use std::path::PathBuf;

use kasa_core::ai_slop::indexers::get_prompt_tags_from_ids_batch;
use sqlx::{query_scalar, sqlite::SqlitePoolOptions, Pool, Sqlite};

pub async fn index_all_ai_images(db_path: &str, max_tag_len: usize) {
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(db_path)
        .await
        .unwrap();

    let all_ids: Vec<String> = query_scalar("SELECT hash FROM Media")
        .fetch_all(&pool)
        .await
        .unwrap();

    get_prompt_tags_from_ids_batch(all_ids, max_tag_len, &pool).await;
}
