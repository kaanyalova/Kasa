use kasa_core::{db::schema::Media, tags::search::SearchCriteria};
use log::trace;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

/// A store containing "extra" search parameters, like those come from TagPicker, or a future sort element
#[derive(Debug, Default)]
pub struct SearchState(Mutex<SearchCriteria>);

use crate::db::{DbStore, MediaCache};

#[tauri::command(async)]
#[specta::specta]
/// `input_raw`: user tags
/// `width`: viewport width for layout
/// `gaps`: gaps between images  
pub async fn search(handle: AppHandle, input_raw: String) {
    // TODO remove width and gaps
    //
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await;

    let search_state = handle.state::<SearchState>();
    let search_guard = search_state.0.lock().await;

    if let Some(pool) = connection_guard.as_ref() {
        let mut search_criteria = SearchCriteria::parse_from_str(&input_raw);

        search_criteria.merge(&search_guard);

        let mut query = search_criteria.to_query();
        let media: Vec<Media> = query.build_query_as().fetch_all(pool).await.unwrap();
        //let media = (&input_raw, pool).await;

        //let rows = calculate_layout(media, width as f64, 0, gaps); // TODO get from config

        let state = handle.state::<MediaCache>();

        *state.media.lock().await = Some(media);

        handle.emit("cache_updated", "").unwrap();
        trace!("cache_updated via search");
    }
}

/// Called when the search store con
#[tauri::command(async)]
#[specta::specta]
pub async fn set_search_store(handle: AppHandle, search_criteria: SearchCriteria) {
    let search_state = handle.state::<SearchState>();
    let mut search_guard = search_state.0.lock().await;

    *search_guard = search_criteria;

    handle.emit("cache_updated", "").unwrap();
    trace!("cache_updated via search");
}
