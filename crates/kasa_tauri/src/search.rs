use kasa_core::{
    db::schema::Media,
    tags::{presets::new_or_update_preset_impl, search::SearchCriteria},
};
use log::trace;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

/// A store containing "extra" search parameters, like those come from TagPicker, or a future sort element
#[derive(Debug, Default)]
pub struct SearchState(Mutex<SearchCriteria>);

use crate::db::{DatabaseState, DbStore, MediaCache};

#[tauri::command(async)]
#[specta::specta]
/// `input_raw`: user tags
/// `width`: viewport width for layout
/// `gaps`: gaps between images  
pub async fn search(handle: AppHandle, input_raw: String) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    let mut search_criteria = SearchCriteria::parse_from_str(&input_raw);

    let search_state = handle.state::<SearchState>();
    let search_guard = search_state.0.lock().await;

    // merge the search inputs from the tag selection and the input box
    search_criteria.merge(&search_guard);

    let mut media: Vec<Media> = vec![];

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(pool) = db_store.db.as_ref() {
                trace!("Searching with criteria {:?}", search_criteria);

                let mut query = search_criteria.to_query();
                media = query.build_query_as().fetch_all(pool).await.unwrap();
                //let media = (&input_raw, pool).await;

                //let rows = calculate_layout(media, width as f64, 0, gaps); // TODO get from config
            }
        }
        DbStore::Remote(remote_store) => {
            media = remote_store.client.search(&search_criteria).await.unwrap()
        }
    }

    trace!("search returned {} items", media.len());
    // update the cache
    let state = handle.state::<MediaCache>();
    *state.media.lock().await = Some(media);
    handle.emit("cache_updated", "").unwrap();
    trace!("cache_updated via search");
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

pub async fn new_or_update_preset(
    handle: AppHandle,
    includes: Vec<String>,
    excludes: Vec<String>,
    name: &str,
) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(pool) = db_store.db.as_ref() {
                let result = new_or_update_preset_impl(includes, excludes, name, pool).await;

                if result.is_err() {
                    trace!("Failed to update preset: {:?}", result);
                } else {
                    trace!("Preset updated: {}", name);
                }
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}
