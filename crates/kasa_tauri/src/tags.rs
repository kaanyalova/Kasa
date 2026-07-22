use kasa_core::{
    config::global_config::get_tag_extractors_dir,
    tags::{
        AllTagsOrderingCriteria, TagWithCount, get_list_of_all_tags_with_details_impl,
        get_tags_as_text_impl, remove_tags, update_tags_impl,
    },
};
use kasa_python::extractors::scriptable::PythonTagExtractor;
use log::trace;
use tauri::{AppHandle, Emitter, Manager};

use crate::db::{DatabaseState, DbStore};

#[tauri::command(async)]
#[specta::specta]
pub async fn update_tags(handle: AppHandle, raw_input: String, hash: String) {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                update_tags_impl(&raw_input, hash, pool).await;
            } else {
                println!("DB connection wasn't initialized yet!")
            }
        }
        DbStore::Remote(remote_store) => remote_store
            .client
            .update_tags(&raw_input, &hash)
            .await
            .unwrap(),
    }

    handle.emit("tags_updated", "").unwrap();
    trace!("Tags updated");
}

#[tauri::command(async)]
#[specta::specta]
pub async fn delete_tags(handle: AppHandle, hash: String, tags: Vec<String>) {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                remove_tags(tags, pool, Some(hash)).await;
            } else {
                println!("DB connection wasn't initialized yet!")
            }
        }
        DbStore::Remote(remote_store) => {
            remote_store.client.delete_tags(&hash, tags).await.unwrap()
        }
    }

    handle.emit("tags_updated", "").unwrap();
    trace!("Tags deleted");
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags_as_text(handle: AppHandle, hash: String) -> Option<String> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let text = get_tags_as_text_impl(&hash, pool).await;
                Some(text)
            } else {
                println!("DB connection wasn't initialized yet!");
                None
            }
        }
        DbStore::Remote(remote_store) => remote_store.client.get_tags_as_text(&hash).await.unwrap(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_list_of_all_tags_with_details(
    handle: AppHandle,
    ordering_criteria: AllTagsOrderingCriteria,
) -> Option<Vec<TagWithCount>> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let tags = get_list_of_all_tags_with_details_impl(pool, ordering_criteria).await;
                Some(tags)
            } else {
                println!("DB connection wasn't initialized yet!");
                None
            }
        }
        DbStore::Remote(remote_store) => Some(
            remote_store
                .client
                .get_list_of_all_tags_with_details(ordering_criteria)
                .await
                .unwrap(),
        ),
    }
}
