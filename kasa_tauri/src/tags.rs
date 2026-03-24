use std::sync::Arc;

use kasa_core::{
    config::global_config::get_tag_extractors_dir,
    tags::{
        AllTagsOrderingCriteria, TagWithCount, get_list_of_all_tags_with_details_impl,
        get_tags_as_text_impl, remove_tags, update_tags_impl,
    },
};
use kasa_python::extractors::scriptable::{PythonTagExtractor, ScriptableTagExtractor};
use log::trace;
use tauri::{AppHandle, Emitter, Manager};

use crate::{db::DbStore, downloaders::PythonStore};

#[tauri::command(async)]
#[specta::specta]
pub async fn update_tags(handle: AppHandle, raw_input: String, hash: String) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        update_tags_impl(&raw_input, hash, pool).await;
        handle.emit("tags_updated", "").unwrap();
        trace!("Tags updated");
    } else {
        println!("DB connection wasn't initialized yet!")
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn delete_tags(handle: AppHandle, hash: String, tags: Vec<String>) {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();
    if let Some(pool) = connection_guard.as_ref() {
        remove_tags(tags, pool, Some(hash)).await;
        handle.emit("tags_updated", "").unwrap();
        trace!("Tags deleted");
    } else {
        println!("DB connection wasn't initialized yet!")
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags_as_text(handle: AppHandle, hash: String) -> Option<String> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();
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
    let connection_guard = connection_state.db.lock().await.clone();
    if let Some(pool) = connection_guard.as_ref() {
        let tags = get_list_of_all_tags_with_details_impl(pool, ordering_criteria).await;
        Some(tags)
    } else {
        println!("DB connection wasn't initialized yet!");
        None
    }
}

pub struct ScriptableTagExtractorStore(PythonTagExtractor);

impl ScriptableTagExtractorStore {
    pub async fn init(handle: AppHandle) -> Self {
        let extractors_dir = get_tag_extractors_dir().unwrap();
        let interpreter = handle.state::<PythonStore>().tagger_interpreter.clone();

        ScriptableTagExtractorStore(PythonTagExtractor::init(interpreter, &extractors_dir).unwrap())
    }
}
