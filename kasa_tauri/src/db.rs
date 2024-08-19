use tokio::sync::Mutex;

use kasa_core::{
    config::global_config::get_config_impl,
    db::{
        self,
        db::{query_all_test_impl, query_tags_impl},
        schema::{Image, Media, Tag},
    },
    layout::google_photos::{calculate_layout, ImageRow},
};
use sqlx::{pool, query, query_as, query_scalar, sqlite::SqlitePoolOptions, Pool, Sqlite};
use tauri::{async_runtime::handle, App, AppHandle, Manager};

#[derive(Default)]
pub struct DbStore {
    pub db: Mutex<Option<Pool<Sqlite>>>,
    pub thumbs_db: Mutex<Option<Pool<Sqlite>>>,
}

#[derive(Default)]
pub struct MediaCache {
    pub media: Mutex<Option<Vec<Media>>>,
}

async fn update_row_cache(handle: AppHandle, media: &Vec<Media>) {
    let state = handle.state::<MediaCache>();
    *state.media.lock().await = Some(media.clone());
}

async fn clean_row_cache(handle: AppHandle) {
    let state = handle.state::<MediaCache>();
    *state.media.lock().await = None;
}

#[tauri::command]
pub async fn get_media_from_cache(handle: AppHandle) -> Option<Vec<Media>> {
    let state = handle.state::<MediaCache>();
    let media = state.media.lock().await;
    media.clone()
}

#[tauri::command]
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

#[tauri::command]
#[specta::specta]
pub async fn query_tags(
    tag_name: String,
    count: i64,
    handle: AppHandle,
) -> Result<Option<Vec<Tag>>, ()> {
    println!("querying tags!");
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let tags = query_tags_impl(tag_name, count, pool).await;
        return Ok(Some(tags));
    } else {
        return Ok(None);
    }
}

#[tauri::command]
#[specta::specta]

pub async fn are_dbs_mounted(handle: AppHandle) -> bool {
    let connection_state = handle.state::<DbStore>();

    let db_connection_guard = connection_state.db.lock().await;
    let thumbs_connection_guard = connection_state.thumbs_db.lock().await;

    db_connection_guard.as_ref().is_some() && thumbs_connection_guard.as_ref().is_some()
}

#[tauri::command]
#[specta::specta]
pub async fn connect_dbs() {
    let config = get_config_impl();

    let pool_db = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&config.db.db_path)
        .await
        .unwrap();

    let pool_thumbs = SqlitePoolOptions::new()
        .max_connections(6)
        .connect(&config.db.db_path)
        .await
        .unwrap();
}

/*
#[tauri::command]
pub async fn query_all(
    page: i64,
    count: i64,
    handle: AppHandle,
) -> Result<Option<Vec<TestMedia>>, ()> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let media = query_all_test_impl(count, page, pool).await;

        dbg!(&media);
        return Ok(Some(media));
    } else {
        Ok(None)
    }
}
*/

#[tauri::command]
#[specta::specta]
pub async fn get_layout_from_cache(
    handle: AppHandle,
    width: f64,
    img_height: u64,
    gaps: u64,
) -> Option<Vec<ImageRow>> {
    let cache = handle.state::<MediaCache>().media.lock().await.clone(); // TODO: lots of clones here , somehow remove them?

    if let Some(media) = cache {
        return Some(calculate_layout(media, width, img_height, gaps));
    } else {
        dbg!("No media found on cache!");
        return None;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn query_all(
    handle: AppHandle,
    width: f64,
    img_height: u64,
    gaps: u64,
) -> Option<Vec<ImageRow>> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let q: Vec<Media> = query_as("SELECT * FROM Media")
            .fetch_all(pool)
            .await
            .unwrap();

        let r = Some(calculate_layout(q.clone(), width, img_height, gaps));

        // Cache the value to MediaCache, to be used for layout calculations
        let state = handle.state::<MediaCache>();
        *state.media.lock().await = Some(q);

        return r;
    } else {
        println!("db was not initialized yet");
        None
    }
}

/*
#[tauri::command]
#[allow(dead_code)]
pub async fn query_all_grouped(
    page: i64,
    group_count: i64,
    item_count: i64,
    handle: AppHandle,
) -> Result<Option<Vec<Vec<TestMedia>>>, ()> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let _media = query_all_test_impl(item_count * group_count, page, pool).await;
        let media: Vec<Vec<TestMedia>> = _media
            .chunks(item_count as usize)
            .map(|s| s.into())
            .collect();

        dbg!(&media);
        return Ok(Some(media));
    } else {
        Ok(None)
    }
}

    */
