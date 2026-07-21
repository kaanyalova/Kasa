use std::path::PathBuf;

use axum::Router;

use kasa_core::{
    config::global_config::{GlobalConfig, get_config_impl},
    db::migrations::{prepare_main_db, prepare_thumbs_db},
    downloaders::download_queue::{
        Downloader, DownloaderContext, DownloaderStateUpdate, init_extractors,
    },
};
use kasa_python::GalleryDlStatus;
use libsqlite3_sys::sqlite3_auto_extension;
use sqlite_vec::sqlite3_vec_init;
use sqlx::{Sqlite, SqlitePool, sqlite::SqliteConnectOptions};
use tokio::sync::broadcast;
use tracing::info;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::api::downloader::DownloaderState;

use crate::{
    api::{Databases, run, write_openapi_spec},
    cli::Args,
};
use clap::Parser;
mod api;
mod cli;
mod simple_auth;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let kasa_config = get_config_impl();
    tracing_subscriber::fmt::init();
    dotenvy::dotenv().ok();

    match args {
        Args::StartServer(server_args) => {
            // if a path is provided use that, otherwise use the last opened db from the client config file
            let db_path = server_args
                .db_path
                .clone()
                .unwrap_or(PathBuf::from(&kasa_config.db.db_path.clone()));

            let thumbs_db_path = server_args
                .thumbs_db_path
                .clone()
                .unwrap_or(PathBuf::from(&kasa_config.thumbs.thumbs_db_path.clone()));

            unsafe {
                sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
            }

            let pool = SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true),
            )
            .await
            .unwrap();

            prepare_main_db(&db_path.to_string_lossy()).await;

            let thumbs_pool = SqlitePool::connect_with(
                SqliteConnectOptions::new()
                    .filename(&thumbs_db_path)
                    .create_if_missing(true),
            )
            .await
            .unwrap();

            prepare_thumbs_db(&thumbs_db_path.to_string_lossy()).await;

            info!("Connected to database at {}", &db_path.display());
            info!(
                "Connected to thumbnail database at {}",
                &thumbs_db_path.display(),
            );

            let config = get_config_impl();

            // set up the callbacks and channels for the downloader
            let (update_tx, _) = broadcast::channel(32);

            let update_tx_progress = update_tx.clone();

            let on_progress = move |status: &GalleryDlStatus| {
                let _ = update_tx_progress
                    .send(DownloaderStateUpdate::OnProgress(status.clone()))
                    .unwrap();
            };

            let update_tx_done = update_tx.clone();

            let on_done = move |hash: String| {
                update_tx_done
                    .send(DownloaderStateUpdate::OnDone(hash))
                    .unwrap();
            };

            let (mut downloader, tx_download_job) = Downloader::init(
                pool.clone(),
                thumbs_pool.clone(),
                config,
                on_progress,
                on_done,
            );
            let extractors = init_extractors(
                downloader.interpreters.tagger_interpreter.clone(),
                &kasa_config,
            )
            .unwrap();

            downloader.set_extractors(extractors);

            // this isn't used anywhere in the server yet
            let downloader_context = DownloaderContext::new();

            tokio::spawn(async move {
                downloader.run(&downloader_context).await;
            });

            let downloader_state = DownloaderState {
                job_tx: tx_download_job,
                update_broadcast: update_tx,
            };

            let dbs = Databases {
                db: pool,
                thumbs_db: thumbs_pool,
            };
            run(&server_args, dbs, downloader_state).await;
        }
        Args::WriteOpenApiSpec(open_api_args) => {
            write_openapi_spec(open_api_args.path.as_deref()).await;
        }
    }
}
