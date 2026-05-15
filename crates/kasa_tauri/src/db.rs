use libsqlite3_sys::sqlite3_auto_extension;
use log::{error, info};
use sqlite_vec::sqlite3_vec_init;
use tokio::sync::Mutex;

use kasa_core::{
    config::global_config::get_config_impl,
    db::{
        db_info::{ThumbsDBInfo, get_thumbs_db_info_impl},
        migrations::prepare_dbs,
        schema::Media,
        {TagQueryOutput, query_tags_impl},
    },
    layout::google_photos::{ImageRow, MediaLayoutData, calculate_layout},
};
use sqlx::{
    Pool, Sqlite, query,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
#[derive(Default)]
pub struct DbStore {
    pub db: Mutex<Option<Pool<Sqlite>>>,
    pub thumbs_db: Mutex<Option<Pool<Sqlite>>>,
}

#[derive(Default)]
pub struct MediaCache {
    pub media: Mutex<Option<Vec<Media>>>,
}

#[tauri::command(async)]
#[specta::specta]
pub async fn connect_to_db(db_path: String, handle: AppHandle) -> Result<(), ()> {
    let options = SqliteConnectOptions::from_str(&db_path)
        .unwrap()
        .pragma("journal_mode", "WAL")
        .pragma("synchronous", "NORMAL")
        .pragma("busy_timeout", "5000");

    let pool = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(options)
        .await
        .unwrap();

    let db_state = handle.state::<DbStore>();
    *db_state.db.lock().await = Some(pool);

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn query_tags(tag_name: String, count: i64, handle: AppHandle) -> Vec<TagQueryOutput> {
    println!("querying tags!");
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        query_tags_impl(tag_name, count, pool).await
    } else {
        error!("no db found when querying tags");
        vec![]
    }
}

#[tauri::command(async)]
#[specta::specta]

pub async fn are_dbs_mounted(handle: AppHandle) -> bool {
    let connection_state = handle.state::<DbStore>();

    let db_connection_guard = connection_state.db.lock().await.clone();
    let thumbs_connection_guard = connection_state.thumbs_db.lock().await.clone();

    db_connection_guard.as_ref().is_some() && thumbs_connection_guard.as_ref().is_some()
}

#[tauri::command(async)]
#[specta::specta]
/// Mounts the dbs into db_store, runs any pending migrations
pub async fn connect_dbs(handle: AppHandle) {
    let config = get_config_impl();

    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    prepare_dbs(&config).await;

    // WARNING ON DEVELOPMENT this causes different path outputs when using the cli and
    // the tauri app, tauri seems to have ./kasa_tauri as its base directory while
    // kasa_cli_utils have ./ as its base dir. Don't use the cli without --db-path
    // if you have something like ../dev.kasa in your config.toml or it will create
    // the db at the parent dir of this repo
    let db_path_absolute = std::path::absolute(&config.db.db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let thumbs_path_absolute = std::path::absolute(&config.thumbs.thumbs_db_path)
        .unwrap()
        .to_string_lossy()
        .to_string();

    let db_options = SqliteConnectOptions::from_str(&db_path_absolute)
        .unwrap()
        .pragma("journal_mode", "WAL")
        .pragma("synchronous", "NORMAL");

    let thumbs_options = SqliteConnectOptions::from_str(&thumbs_path_absolute)
        .unwrap()
        .pragma("journal_mode", "WAL")
        .pragma("synchronous", "NORMAL");

    let pool_db = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(db_options)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(32)
        .connect_with(thumbs_options)
        .await
        .unwrap();

    // mount the dbs
    let db_store = handle.state::<DbStore>();
    *db_store.db.lock().await = Some(pool_db);
    *db_store.thumbs_db.lock().await = Some(pool_thumbs);
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_layout_from_cache(
    handle: AppHandle,
    width: f64,
    gaps: u64,
    scale: f64,
) -> Option<Vec<ImageRow>> {
    let cache = handle.state::<MediaCache>().media.lock().await.clone(); // TODO: lots of clones here , somehow remove them?

    if let Some(media) = cache {
        let layout_data = media
            .into_iter()
            .map(|m| MediaLayoutData {
                hash: m.hash,
                thumbnail_x: m.thumbnail_x as i64,
                thumbnail_y: m.thumbnail_y as i64,
            })
            .collect();
        Some(calculate_layout(layout_data, scale, width, gaps))
    } else {
        info!("No media found on cache!");
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_thumbs_db_info(handle: AppHandle) -> Option<ThumbsDBInfo> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.thumbs_db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        Some(get_thumbs_db_info_impl(pool).await)
    } else {
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn nuke_db_versioning(handle: AppHandle) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.thumbs_db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        query("DROP TABLE _sqlx_migrations")
            .execute(pool)
            .await
            .unwrap();
    } else {
        error!("Cannot connect to the db");
    }
}
