use kasa_core::{
    db::schema::Media,
    layout::google_photos::{calculate_layout, ImageRow},
    tags::search::search_simple_impl,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::{DbStore, MediaCache};

#[tauri::command]
#[specta::specta]
/// `input_raw`: user tags
/// `width`: viewport width for layout
/// `gaps`: gaps between images  
pub async fn search(handle: AppHandle, input_raw: String, width: u64, gaps: u64) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let media = search_simple_impl(&input_raw, pool).await;

        //let rows = calculate_layout(media, width as f64, 0, gaps); // TODO get from config

        let state = handle.state::<MediaCache>();

        *state.media.lock().await = Some(media);

        handle.emit("media_updated", "").unwrap();
    }
}
