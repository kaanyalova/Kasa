use kasa_core::tags::{
    AllTagsOrderingCriteria, TagWithCount, get_list_of_all_tags_with_details_impl,
    get_tags_as_text_impl, remove_tags, update_tags_impl,
};
use tauri::{AppHandle, Manager};

use crate::db::DbStore;

#[tauri::command(async)]
#[specta::specta]
pub async fn update_tags(handle: AppHandle, raw_input: String, hash: String) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        update_tags_impl(&raw_input, hash, pool).await;
    } else {
        println!("DB connection wasn't initialized yet!")
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn delete_tags(handle: AppHandle, hash: String, tags: Vec<String>) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        remove_tags(tags, pool, Some(hash)).await;
    } else {
        println!("DB connection wasn't initialized yet!")
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags_as_text(handle: AppHandle, hash: String) -> Option<String> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let text = get_tags_as_text_impl(&hash, pool).await;
        Some(text)
    } else {
        println!("DB connection wasn't initialized yet!");
        None
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_list_of_all_tags_with_details(
    handle: AppHandle,
    ordering_criteria: AllTagsOrderingCriteria,
) -> Option<Vec<TagWithCount>> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let tags = get_list_of_all_tags_with_details_impl(pool, ordering_criteria).await;
        Some(tags)
    } else {
        println!("DB connection wasn't initialized yet!");
        None
    }
}
