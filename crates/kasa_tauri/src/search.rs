use kasa_core::{
    db::schema::Media,
    tags::{presets::new_or_update_preset_impl, search::SearchCriteria},
};
use log::trace;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::sync::Mutex;

/// A store containing "extra" search parameters, like those come from TagPicker, or a future sort element
#[derive(Debug, Default)]
pub struct SearchState {
    pub input: Mutex<String>,
    pub criteria: Mutex<SearchCriteria>,
}

use crate::{
    db::{DatabaseState, DbStore, MediaCache},
    events::CacheUpdatedEvent,
};

async fn search_impl(handle: AppHandle, reload_virtual_list: bool) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    let search_state = handle.state::<SearchState>();
    let input = search_state.input.lock().await.clone();
    let search_guard = search_state.criteria.lock().await;

    let mut search_criteria = SearchCriteria::parse_from_str(&input);

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

    CacheUpdatedEvent {
        reload_virtual_list,
    }
    .emit(&handle)
    .unwrap();
    trace!("cache_updated via search");
}

#[tauri::command(async)]
#[specta::specta]
pub async fn search(handle: AppHandle) {
    search_impl(handle, false).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn search_and_reload(handle: AppHandle) {
    search_impl(handle, true).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn set_search_criteria(handle: AppHandle, search_criteria: SearchCriteria) {
    let search_state = handle.state::<SearchState>();
    let mut search_guard = search_state.criteria.lock().await;

    *search_guard = search_criteria;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn set_search_input(handle: AppHandle, input: String) {
    let search_state = handle.state::<SearchState>();
    let mut search_guard = search_state.input.lock().await;

    *search_guard = input;
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
