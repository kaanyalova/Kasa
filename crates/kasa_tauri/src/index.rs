use kasa_core::index::{
    index_sources::{
        add_index_source_impl, cleanup_unreferenced_files_impl, get_index_paths_impl,
        index_all_impl, nuke_all_indexes_impl, nuke_selected_index_impl, remove_index_source_impl,
    },
    indexer::index,
};
use tauri::{AppHandle, Manager};

use crate::{
    db::{DatabaseState, DbStore},
    search::search,
};

#[tauri::command(async)]
#[specta::specta]

/// Adds a single index source from the path, does not index that path without calling index_path()
pub async fn add_index_source(handle: AppHandle, path: String) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(db) = db_store.db.as_ref() {
                add_index_source_impl(&path, db).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn remove_index_source(handle: AppHandle, path: String) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(db) = db_store.db.as_ref() {
                remove_index_source_impl(&path, db).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn index_all(handle: AppHandle) -> Result<(), ()> {
    let h = handle.clone();
    let db_store = h.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(db), Some(thumbs_db)) = (db_store.db.as_ref(), db_store.thumbs_db.as_ref())
            {
                index_all_impl(db, thumbs_db).await;

                search(handle.clone()).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }

    Ok(())
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_index_paths(handle: AppHandle) -> Vec<String> {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let Some(db) = db_store.db.as_ref() {
                get_index_paths_impl(db).await
            } else {
                vec![]
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn index_path(handle: AppHandle, path: String) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(db), Some(thumbs)) = (db_store.db.as_ref(), db_store.thumbs_db.as_ref()) {
                index(&path, db, thumbs).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }

    search(handle.clone()).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn nuke_selected_index(handle: AppHandle, path: String) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(db), Some(thumbs)) = (db_store.db.as_ref(), db_store.thumbs_db.as_ref()) {
                nuke_selected_index_impl(db, Some(thumbs), &path).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }

    search(handle.clone()).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn nuke_all_indexes(handle: AppHandle) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(db), Some(thumbs)) = (db_store.db.as_ref(), db_store.thumbs_db.as_ref()) {
                nuke_all_indexes_impl(db, Some(thumbs)).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }

    search(handle.clone()).await;
}

#[tauri::command(async)]
#[specta::specta]
pub async fn cleanup_unreferenced_files(handle: AppHandle) {
    let db_store = handle.state::<DatabaseState>().clone_store().await;

    match db_store {
        DbStore::Local(db_store) => {
            if let (Some(db), Some(thumbs)) = (db_store.db.as_ref(), db_store.thumbs_db.as_ref()) {
                cleanup_unreferenced_files_impl(db, thumbs).await;
            }
        }
        DbStore::Remote(_) => todo!(),
    }

    search(handle.clone()).await;
}
