use kasa_core::tags::tags::update_tags_impl;
use tauri::{AppHandle, Manager};

use crate::db::DbStore;

#[tauri::command]
#[specta::specta]
pub async fn update_tags(handle: AppHandle, raw_input: String, hash: String) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        update_tags_impl(&raw_input, &hash, pool).await;
    } else {
        println!("DB connection wasn't initialized yet!")
    }
}
