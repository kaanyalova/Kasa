use crate::db::DbStore;
use kasa_core::groups::get_group_info_impl;
use kasa_core::media::{
    MediaInfo, SourceCategoryGroupedTags, TagWithDetails, get_info_impl, get_media_name_impl,
    get_media_type_impl, get_tags_detailed_impl, get_tags_grouped_by_source_categories_impl,
};
use kasa_core::thumbnail::thumbnail_flash::get_flash_resolution_impl;
use log::error;
use tauri::{AppHandle, Manager};

#[tauri::command(async)]
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

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags(handle: AppHandle, hash: String) -> Option<Vec<TagWithDetails>> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let tags = get_tags_detailed_impl(&hash, pool).await;
        Some(tags)
    } else {
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_media_type(handle: AppHandle, hash: String) -> String {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        get_media_type_impl(&hash, pool).await
    } else {
        "".to_string()
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_swf_resolution(path: String) -> (u32, u32) {
    get_flash_resolution_impl(&path).unwrap()
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_group_info(handle: AppHandle, group_hash: String) -> Vec<MediaInfo> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        match get_group_info_impl(pool, &group_hash).await {
            Ok(info) => info,
            Err(e) => {
                error!("Error getting group info: {}", e);
                vec![]
            }
        }
    } else {
        error!("No connection to database , could not get group info");
        vec![]
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags_grouped_by_source_categories(
    handle: AppHandle,
    hash: String,
) -> Option<SourceCategoryGroupedTags> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let tags = get_tags_grouped_by_source_categories_impl(&hash, pool).await;
        Some(tags)
    } else {
        error!("No connection to database , could not get group info");
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_media_name(handle: AppHandle, hash: String) -> String {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        get_media_name_impl(&hash, pool).await
    } else {
        error!("No connection to database , could not get group info");
        "".to_string()
    }
}
