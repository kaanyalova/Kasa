use log::{error, info};
use tokio::sync::Mutex;

use kasa_core::{
    config::global_config::get_config_impl,
    db::{
        db_info::{ThumbsDBInfo, get_thumbs_db_info_impl},
        migrations::prepare_dbs,
        schema::Media,
        {TagQueryOutput, query_tags_impl},
    },
    layout::google_photos::{ImageRow, calculate_layout},
};
use sqlx::{Pool, Sqlite, query, sqlite::SqlitePoolOptions};
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
    let pool = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&db_path)
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
    let connection_guard = connection_state.db.lock().await;

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

    let db_connection_guard = connection_state.db.lock().await;
    let thumbs_connection_guard = connection_state.thumbs_db.lock().await;

    db_connection_guard.as_ref().is_some() && thumbs_connection_guard.as_ref().is_some()
}

#[tauri::command(async)]
#[specta::specta]
/// Mounts the dbs into db_store, runs any pending migrations
pub async fn connect_dbs(handle: AppHandle) {
    let config = get_config_impl();

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

    let pool_db = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&db_path_absolute)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&thumbs_path_absolute)
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
    img_height: u64,
    gaps: u64,
) -> Option<Vec<ImageRow>> {
    let cache = handle.state::<MediaCache>().media.lock().await.clone(); // TODO: lots of clones here , somehow remove them?

    if let Some(media) = cache {
        Some(calculate_layout(media, width, img_height, gaps))
    } else {
        info!("No media found on cache!");
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_thumbs_db_info(handle: AppHandle) -> Option<ThumbsDBInfo> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.thumbs_db.lock().await;

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
    let connection_guard = connection_state.thumbs_db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        query("DROP TABLE _sqlx_migrations")
            .execute(pool)
            .await
            .unwrap();
    } else {
        error!("Cannot connect to the db");
    }
}
