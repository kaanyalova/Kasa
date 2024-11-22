use std::{collections::HashMap, path::Path};

use anyhow::Result;
use kasa_python::{extractors::configurable::ExtractorConfig, gdl_download, init_interpreter};
use rustpython_vm::Interpreter;
use serde_json::error;
use sqlx::{query, query_scalar, Pool, Sqlite};
use thiserror::Error;

use crate::{
    config::global_config::{get_config_impl, get_configurable_tag_extractor_path},
    index::indexer::index,
    tags::tags::insert_tags_with_source_types,
};

use tokio::sync::oneshot;

// fuck...
pub struct PyTrustMe(pub Interpreter);
unsafe impl Send for PyTrustMe {}
unsafe impl Sync for PyTrustMe {}

/// output_path should be an absolute path
pub async fn download_and_index_impl<'py, F: Fn() + Send + Sync>(
    interpreter: &PyTrustMe,
    url: &str,
    output_path: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
    when_done: F,
    extractors: &HashMap<String, ExtractorConfig>,
) -> Result<()> {
    let config = get_config_impl();

    if !Path::new(output_path).is_absolute() {
        return Err(DownloaderError::NotAnAbsolutePath.into());
    }

    let downloader_output = kasa_python::gdl_download(
        &interpreter.0,
        url,
        output_path,
        Some(config.downloader.gdl_config_path),
    )?;

    for extractor in downloader_output.url_extractors {
        index(&extractor.path, pool, pool_thumbs).await;

        let hash: String = query_scalar("SELECT * FROM Path WHERE path = ?")
            .bind(&extractor.path)
            .fetch_one(pool)
            .await?;

        //dbg!(&extractor.get_tags());

        insert_tags_with_source_types(extractor.get_tags(&extractors)?, pool, Some(hash), None)
            .await;
    }

    when_done();
    Ok(())
}

#[derive(Error, Debug)]
enum DownloaderError {
    #[error("The provided path should be absolute")]
    NotAnAbsolutePath,
}
