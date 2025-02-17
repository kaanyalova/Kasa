use kasa_core::{
    db::schema::Media,
    tags::search::{search_simple_impl, SearchCriteria},
};
use tauri::{AppHandle, Emitter, Manager};

use crate::db::{DbStore, MediaCache};

#[tauri::command(async)]
#[specta::specta]
/// `input_raw`: user tags
/// `width`: viewport width for layout
/// `gaps`: gaps between images  
pub async fn search(handle: AppHandle, input_raw: String, _width: u64, _gaps: u64) {
    // TODO remove width and gaps
    //
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;
    if let Some(pool) = connection_guard.as_ref() {
        let search_criteria = SearchCriteria::parse_from_str(&input_raw);

        let mut query = search_criteria.to_query();
        let media: Vec<Media> = query.build_query_as().fetch_all(pool).await.unwrap();
        //let media = (&input_raw, pool).await;

        //let rows = calculate_layout(media, width as f64, 0, gaps); // TODO get from config

        let state = handle.state::<MediaCache>();

        *state.media.lock().await = Some(media);

        handle.emit("cache_updated", "").unwrap();
    }
}
