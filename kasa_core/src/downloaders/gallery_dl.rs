use std::path::Path;

use anyhow::Result;
use rustpython_vm::Interpreter;
use serde_json::error;
use sqlx::{query, query_scalar, Pool, Sqlite};
use thiserror::Error;

use crate::{index::indexer::index, tags::tags::insert_tags_with_source_types};

/// output_path should be an absolute path
pub async fn download_and_index(
    interpreter: Interpreter,
    url: &str,
    output_path: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
) -> Result<()> {
    if !Path::new(output_path).is_absolute() {
        return Err(DownloaderError::NotAnAbsolutePath.into());
    }

    let downloader_output = kasa_python::gdl_download(interpreter, url, output_path)?;

    for extractor in downloader_output.url_extractors {
        index(&extractor.path, pool, pool_thumbs).await;

        let hash: String = query_scalar("SELECT * FROM Path WHERE path = ?")
            .bind(&extractor.path)
            .fetch_one(pool)
            .await?;

        dbg!(&extractor.get_tags());

        insert_tags_with_source_types(extractor.get_tags(), pool, Some(hash), None).await;
    }

    Ok(())
}

#[derive(Error, Debug)]
enum DownloaderError {
    #[error("The provided path should be absolute")]
    NotAnAbsolutePath,
}
