use kasa_core::media::{get_info_impl, get_tags_impl, MediaInfo, MediaTag};
use tauri::{App, AppHandle, Manager};

use crate::db::DbStore;

#[tauri::command]
#[specta::specta]
pub async fn get_info(handle: AppHandle, hash: String) -> Option<MediaInfo> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let i = get_info_impl(&hash, pool).await;
        Some(i)
    } else {
        None
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_tags(handle: AppHandle, hash: String) -> Option<Vec<MediaTag>> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let tags = get_tags_impl(&hash, pool).await;
        Some(tags)
    } else {
        None
    }
}
