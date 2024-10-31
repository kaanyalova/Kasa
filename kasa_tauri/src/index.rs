use kasa_core::index::index_sources::{
    add_index_source_impl, get_index_paths_impl, index_all_impl, remove_index_source_impl,
};
use sqlx::{pool::PoolOptions, Pool, Sqlite};
use tauri::{AppHandle, Manager};

use crate::db::DbStore;

#[tauri::command]
#[specta::specta]
pub async fn add_index_source(handle: AppHandle, path: String) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(db) = connection_guard.as_ref() {
        add_index_source_impl(&path, db).await;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn remove_index_source(handle: AppHandle, path: String) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    let connection_guard_thumbs = connection_state.thumbs_db.lock().await;

    if let Some(db) = connection_guard.as_ref() {
        remove_index_source_impl(&path, db).await;
    }
}

#[tauri::command]
#[specta::specta]
pub async fn index_all(handle: AppHandle) -> Result<(), ()> {
    let h = handle.clone();
    let connection_state = h.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    let connection_guard_thumbs = connection_state.thumbs_db.lock().await;

    if let (Some(db), Some(thumbs_db)) =
        (connection_guard.as_ref(), connection_guard_thumbs.as_ref())
    {
        let p: Pool<Sqlite> = PoolOptions::new().connect("").await.unwrap();
        index_all_impl(&p, &p).await;
        //index_all_impl(db, thumbs_db).await;
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn get_index_paths(handle: AppHandle) -> Vec<String> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(db) = connection_guard.as_ref() {
        get_index_paths_impl(db).await
    } else {
        return vec![];
    }
}
