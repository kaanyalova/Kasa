use std::{
    process,
    sync::{Arc, Mutex},
    vec,
};

use kasa_core::{
    config::global_config::{get_config_impl, get_tag_extractors_dir},
    db::migrations::{prepare_dbs, prepare_dbs_from_config},
    downloaders::{
        download_queue::{DownloaderStateUpdate, job_channel, run as run_downloader},
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

    let on_update = |update: DownloaderStateUpdate| {
        if let DownloaderStateUpdate::OnDone(hash) = update {
            println!("Download done for hash: {}", hash);
            process::exit(0);
        }
    };

    let (_job_tx, rx) = job_channel();

    tokio::spawn(async move {
        if let Err(e) = run_downloader(
            pool.clone(),
            pool_thumbs.clone(),
            config.clone(),
            rx,
            on_update,
        )
        .await
        {
            eprintln!("downloader failed: {e}");
        }
    });
}
