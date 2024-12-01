use std::path::PathBuf;

use kasa_core::{
    config::global_config::get_config_impl,
    index::postprocess::ai_indexer::get_prompt_tags_from_ids_batch,
};
use sqlx::{query_scalar, sqlite::SqlitePoolOptions};

pub async fn index_all_ai_images(db_path: Option<PathBuf>, max_tag_len: usize) {
    let config = get_config_impl();

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(
            &db_path
                .unwrap_or((&config.db.db_path).into())
                .as_os_str()
                .to_string_lossy(),
        )
        .await
        .unwrap();

    let all_ids: Vec<String> = query_scalar("SELECT hash FROM Media")
        .fetch_all(&pool)
        .await
        .unwrap();

    get_prompt_tags_from_ids_batch(all_ids, max_tag_len, &pool).await;
}
