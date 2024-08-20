use std::{os::unix::fs::MetadataExt, path::PathBuf};

use serde::{Deserialize, Serialize};
use sqlx::{query_scalar, Pool, Sqlite};

use crate::config::global_config::get_config_impl;

#[derive(Debug, specta::Type, Serialize, Deserialize)]
pub struct ThumbsDBInfo {
    pub path: String,
    pub size: String,
    pub image_count: i64,
}

pub async fn get_thumbs_db_info_impl(pool_thumbs: &Pool<Sqlite>) -> ThumbsDBInfo {
    let config = get_config_impl();

    let path = config.thumbs.thumbs_db_path;

    let pathbuf = PathBuf::from(&path);
    let file_size = pathbuf.metadata().unwrap().size();
    let file_size_human_readable = human_bytes::human_bytes(file_size as f64);

    let image_count: i64 = query_scalar("SELECT COUNT(*) FROM Thumbs")
        .fetch_one(pool_thumbs)
        .await
        .unwrap();

    ThumbsDBInfo {
        path,
        size: file_size_human_readable,
        image_count,
    }
}
