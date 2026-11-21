use std::default;

use kasa_core::{
    db::schema::Media,
    tags::{
        presets::new_or_update_preset_impl,
        search::{SearchCriteria, SpecialSearch},
    },
};
use log::trace;
use tauri::{AppHandle, Manager};
use tauri_specta::Event;
use tokio::sync::Mutex;

/// A store containing "extra" search parameters, like those come from TagPicker, or a future sort element
#[derive(Debug, Default)]
pub struct SearchState {
    pub params: Mutex<SearchParams>,
}

#[derive(Debug, Default)]
pub struct SearchParams {
    pub input: String,
    pub input_type: SearchType,
    pub criteria: SearchCriteria,
}

#[derive(Debug, Default, specta::Type)]
pub enum SearchType {
    #[default]
    TagSearch,
    OcrTextSearch,
    EmbeddingSearch,
}

use crate::{
    db::{DatabaseState, DbStore, MediaCache},
    events::CacheUpdatedEvent,
};

async fn search_impl(handle: AppHandle, reload_virtual_list: bool) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    let search_state = handle.state::<SearchState>();

    let search_criteria = {
        let params = search_state.params.lock().await;

        // first get the params from the menu
        let mut criteria = params.criteria.clone();

        match params.input_type {
            SearchType::TagSearch => {
                // if we are making a regular tag search parse the tags from the input string then merge it with the info from the tags
                let parsed_text_params = SearchCriteria::parse_tags_from_str(&params.input);
                criteria.merge(&parsed_text_params);
            }
            SearchType::OcrTextSearch => {
                criteria.set_special_search(SpecialSearch::OcrText(params.input.clone()))
            }
            SearchType::EmbeddingSearch => {
                criteria.set_special_search(SpecialSearch::Embedding(params.input.clone()))
            }
        }

        criteria
    };

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
        _ => panic!("db not initialized"),
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
    search_state.params.lock().await.criteria = search_criteria;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn set_search_input(handle: AppHandle, input: String) {
    let search_state = handle.state::<SearchState>();
    search_state.params.lock().await.input = input;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn set_special_search(handle: AppHandle, special_search: SearchType) {
    let search_state = handle.state::<SearchState>();
    search_state.params.lock().await.input_type = special_search;
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
        _ => panic!("db not initialized"),
    }
}
