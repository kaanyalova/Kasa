use std::{path::Path, sync::Arc};

use anyhow::Result;
use kasa_python::{
    extractors::TagExtractor,
    worker::downloader::{DownloadJob, GalleryDlDownloadWorker},
};
use sha1::{Digest, Sha1};
use sqlx::{Pool, Sqlite, query_scalar};
use thiserror::Error;

use sqlx::query;

use crate::{
    config::global_config::get_config_impl, index::indexer::index,
    tags::insert_tags_with_source_types,
};

/// output_path should be an absolute path
pub async fn download_and_index_impl(
    worker: Arc<GalleryDlDownloadWorker>,
    url: &str,
    output_path: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
    when_done: impl Fn(String) + Send + Sync,
    extractors: &Vec<&(dyn TagExtractor + Send + Sync)>,
) -> Result<()> {
    let config = get_config_impl();

    if !Path::new(output_path).is_absolute() {
        return Err(DownloaderError::NotAnAbsolutePath.into());
    }

    let download_result = worker.push_job(
        DownloadJob {
            url: url.to_owned(),
        },
        output_path.clone(),
        config.downloader.gdl_config_path.as_deref(),
    )?;

    let downloader_output = download_result.recv()??;

    for url_extractor in downloader_output.url_extractors {
        index(&url_extractor.path, pool, pool_thumbs).await;

        let hash: String = query_scalar("SELECT * FROM Path WHERE path = ?")
            .bind(&url_extractor.path)
            .fetch_one(pool)
            .await?;

        //dbg!(&extractor.get_tags());

        let raw_data: String = serde_json::to_string(&url_extractor)?;
        query("INSERT OR IGNORE INTO MediaSource(hash, importer_type, link_or_path, source, raw_data) VALUES (?, ?, ?, ?, ?)")
            .bind(&hash)
            .bind("gallery_dl")
            .bind(&url)
            .bind(&downloader_output.extractor)
            .bind(&raw_data)
            .execute(pool)
            .await?;

        insert_tags_with_source_types(
            url_extractor.extract_tags(extractors)?,
            pool,
            Some(hash),
            None,
        )
        .await;
    }

    let url_hash = hash_url(url);
    when_done(url_hash);

    Ok(())
}

pub fn hash_url(url: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(url.as_bytes());
    let result = hasher.finalize();
    hex::encode(result)
}

#[derive(Error, Debug)]
enum DownloaderError {
    #[error("The provided path should be absolute")]
    NotAnAbsolutePath,
}
