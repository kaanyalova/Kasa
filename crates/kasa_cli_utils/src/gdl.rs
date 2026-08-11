use std::{
    process,
    sync::{Arc, Mutex},
    vec,
};

use kasa_core::{
    config::global_config::{get_config_impl, get_tag_extractors_dir},
    db::migrations::{prepare_dbs, prepare_dbs_from_config},
    downloaders::{
        download_queue::{Downloader, DownloaderContext, init_extractors},
        gallery_dl::download_and_index_impl,
    },
};
use kasa_python::{
    extractors::{
        TagExtractor, configurable::ConfigurableExtractor, scriptable::PythonTagExtractor,
    },
    init_interpreter, init_interpreter_with_gallery_dl,
};
use sqlx::sqlite::SqlitePoolOptions;

pub async fn gdl(url: &str) {
    let config = get_config_impl();

    prepare_dbs_from_config(&config).await;

    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(&config.thumbs.thumbs_db_path)
        .await
        .unwrap();

    let (mut downloader, job_tx) = Downloader::init(
        pool.clone(),
        pool_thumbs.clone(),
        config.clone(),
        |_| {},
        |hash| {
            println!("Download done for hash: {}", hash);
            process::exit(0);
        },
    );

    let extractors = init_extractors(downloader.workers.tagger_worker.clone(), &config).unwrap();
    downloader.set_extractors(extractors);

    let downloader_context = DownloaderContext::new();

    tokio::spawn(async move {
        downloader.run(&downloader_context).await;
    });
}
