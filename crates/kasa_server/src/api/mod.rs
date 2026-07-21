mod db;
pub(crate) mod downloader;
mod image;
mod media;
mod media_server;
mod search;
mod tags;

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::mpsc;

use crate::api::db::__path_query_tags;
use crate::api::db::query_tags;
use crate::api::downloader::__path_listen_for_download_updates;
use crate::api::downloader::DownloaderState;
use crate::api::downloader::listen_for_download_updates;
use crate::api::image::__path_get_thumbnail;
use crate::api::image::get_thumbnail;
use crate::api::media::__path_get_info;
use crate::api::media::__path_get_media_name;
use crate::api::media::__path_get_media_sources;
use crate::api::media::__path_get_media_type;
use crate::api::media::__path_get_tags;
use crate::api::media::__path_get_tags_grouped_by_source_categories;
use crate::api::media::__path_get_top_n_closest_for_media;
use crate::api::media::__path_get_video_length;
use crate::api::media::__path_set_media_favorite;
use crate::api::media::get_info;
use crate::api::media::get_media_name;
use crate::api::media::get_media_sources;
use crate::api::media::get_media_type;
use crate::api::media::get_tags;
use crate::api::media::get_tags_grouped_by_source_categories;
use crate::api::media::get_top_n_closest_for_media;
use crate::api::media::get_video_length;
use crate::api::media::set_media_favorite;
use crate::api::media_server::__path_serve_media;
use crate::api::media_server::serve_media;
use crate::api::search::__path_search;
use crate::api::search::search;
use crate::api::tags::__path_delete_tags;
use crate::api::tags::__path_get_list_of_all_tags_with_details;
use crate::api::tags::__path_get_tags_as_text;
use crate::api::tags::__path_update_tags;
use crate::api::tags::delete_tags;
use crate::api::tags::get_list_of_all_tags_with_details;
use crate::api::tags::get_tags_as_text;
use crate::api::tags::update_tags;
use crate::cli::ServerArgs;

use crate::api::downloader::__path_push_download;
use crate::api::downloader::push_download;

use axum::extract::FromRef;
use futures_util::lock::Mutex;
use kasa_core::downloaders::download_queue::Downloader;
use kasa_python::Interpreter;
use sqlx::{Pool, Sqlite};
use tokio::sync::broadcast;
use tower_http::compression::CompressionLayer;
use utoipa_axum::{router::OpenApiRouter, routes};

#[derive(Clone)]
pub struct Databases {
    pub db: Pool<Sqlite>,
    pub thumbs_db: Pool<Sqlite>,
}

#[derive(Debug, Default)]
pub struct ServedMedia {
    pub hash: String,
    pub path: String,
}

#[derive(Clone, FromRef)]
pub struct AppState {
    pub dbs: Databases,
    pub downloader_state: DownloaderState,
}

fn create_router(dbs: Databases, downloader_state: DownloaderState) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(are_dbs_mounted))
        .routes(routes!(query_tags))
        .routes(routes!(get_thumbnail))
        .routes(routes!(get_info))
        .routes(routes!(get_tags))
        .routes(routes!(get_media_type))
        .routes(routes!(get_tags_grouped_by_source_categories))
        .routes(routes!(get_media_name))
        .routes(routes!(get_media_sources))
        .routes(routes!(set_media_favorite))
        .routes(routes!(get_video_length))
        .routes(routes!(get_top_n_closest_for_media))
        .routes(routes!(search))
        .routes(routes!(update_tags))
        .routes(routes!(delete_tags))
        .routes(routes!(serve_media))
        .routes(routes!(get_tags_as_text))
        .routes(routes!(get_list_of_all_tags_with_details))
        .routes(routes!(listen_for_download_updates))
        .routes(routes!(push_download))
        .with_state(AppState {
            dbs,
            downloader_state,
        })
        .layer(CompressionLayer::new())
}

pub async fn get_openapi_spec() -> String {
    let dummy = Pool::<Sqlite>::connect_with(
        sqlx::sqlite::SqliteConnectOptions::new().filename(":memory:"),
    )
    .await
    .unwrap();

    let dbs = Databases {
        db: dummy.clone(),
        thumbs_db: dummy,
    };

    let (job_tx, _) = tokio::sync::mpsc::channel(32);
    let (update_broadcast, _) = broadcast::channel(32);

    let downloader_state = DownloaderState {
        job_tx,
        update_broadcast,
    };

    let (_app, api) = create_router(dbs, downloader_state).split_for_parts();

    api.to_pretty_json().unwrap()
}

pub async fn write_openapi_spec(path: Option<&Path>) {
    let schema = get_openapi_spec().await;

    if let Some(path) = path {
        fs::write(path, schema).unwrap();
    } else {
        println!("{}", schema);
    }
}

pub async fn run(args: &ServerArgs, dbs: Databases, downloader_state: DownloaderState) {
    let (app, _api) = create_router(dbs, downloader_state).split_for_parts();

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", args.ip_address, args.port))
        .await
        .unwrap();
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

#[utoipa::path(
    get,
    path = "/ping",
    responses(
        (status = 200, description = "Server is running", body = str),
    ),
)]
async fn are_dbs_mounted() -> &'static str {
    "pong"
}
