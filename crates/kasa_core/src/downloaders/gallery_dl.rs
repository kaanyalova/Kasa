use std::{path::Path, sync::Arc};

use anyhow::Result;
use kasa_python::{
    GalleryDlStatus, PyTrustMe,
    extractors::TagExtractor,
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
    interpreter: Arc<PyTrustMe>,
    url: &str,
    output_path: &str,
    pool: &Pool<Sqlite>,
    pool_thumbs: &Pool<Sqlite>,
    when_done: impl Fn(String) + Send + Sync,
    on_progress: impl Fn(GalleryDlStatus) + Send + Sync + 'static,
    extractors: &Vec<&(dyn TagExtractor + Send + Sync)>,
) -> Result<()> {
    let config = get_config_impl();

    if !Path::new(output_path).is_absolute() {
        return Err(DownloaderError::NotAnAbsolutePath.into());
    }

    let url_owned = url.to_owned();

    let output_path_owned = output_path.to_owned();

    let (tx, rx) = tokio::sync::oneshot::channel();

    // rustpython stack overflows on debug more with the default 4mbs of stack
    std::thread::Builder::new()
        .name("rustpython".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || {
            let result = kasa_python::gdl_download(
                &interpreter.0,
                &url_owned,
                &output_path_owned,
                config.downloader.gdl_config_path,
                on_progress,
            );

            tx.send(result).unwrap();
        })
        .unwrap();

    let downloader_output = rx.await??;

    //let downloader_output = tokio::task::spawn_blocking(move || {
    //    kasa_python::gdl_download(
    //        &interpreter.0,
    //        &url_owned,
    //        &output_path_owned,
    //        config.downloader.gdl_config_path,
    //        on_progress,
    //    )
    //})
    //.await
    //.unwrap()
    //.unwrap();

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
