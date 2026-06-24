use crate::db::{DatabaseState, DbStore};
use kasa_core::db::embeddings::{EmbeddingDistance, get_top_n_closest_for_media_impl};
use kasa_core::db::schema::MediaSource;
use kasa_core::groups::get_group_info_impl;
use kasa_core::media::{
    MediaInfo, SourceCategoryGroupedTags, TagWithDetails, get_info_impl, get_media_name_impl,
    get_media_sources_impl, get_media_type_impl, get_tags_detailed_impl,
    get_tags_grouped_by_source_categories_impl, set_media_favorite_impl,
};
use kasa_core::thumbnail::thumbnail_flash::get_flash_resolution_impl;
use log::error;
use tauri::{AppHandle, Manager};

#[tauri::command(async)]
#[specta::specta]
pub async fn get_info(handle: AppHandle, hash: String) -> Option<MediaInfo> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let i = get_info_impl(&hash, pool).await;
                Some(i)
            } else {
                None
            }
        }
        DbStore::Remote(remote_store) => remote_store.client.get_info(&hash).await.unwrap(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags(handle: AppHandle, hash: String) -> Option<Vec<TagWithDetails>> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let tags = get_tags_detailed_impl(&hash, pool).await;
                Some(tags)
            } else {
                None
            }
        }
        DbStore::Remote(remote_client) => {
            let tags = remote_client.client.get_tags(&hash).await.unwrap();
            Some(tags)
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_media_type(handle: AppHandle, hash: String) -> String {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                get_media_type_impl(&hash, pool).await
            } else {
                "".to_string()
            }
        }
        DbStore::Remote(remote_store) => remote_store.client.get_media_type(&hash).await.unwrap(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_swf_resolution(path: String) -> (u32, u32) {
    get_flash_resolution_impl(&path).unwrap()
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_group_info(handle: AppHandle, group_hash: String) -> Vec<MediaInfo> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                match get_group_info_impl(pool, &group_hash).await {
                    Ok(info) => info,
                    Err(e) => {
                        error!("Error getting group info: {}", e);
                        vec![]
                    }
                }
            } else {
                error!("No connection to database , could not get group info");
                vec![]
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_tags_grouped_by_source_categories(
    handle: AppHandle,
    hash: String,
) -> Option<SourceCategoryGroupedTags> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let tags = get_tags_grouped_by_source_categories_impl(&hash, pool).await;
                Some(tags)
            } else {
                error!("No connection to database , could not get group info");
                None
            }
        }
        DbStore::Remote(remote_store) => {
            let tags = remote_store
                .client
                .get_tags_grouped_by_source_categories(&hash)
                .await
                .unwrap();
            Some(tags)
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_media_name(handle: AppHandle, hash: String) -> String {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                get_media_name_impl(&hash, pool).await
            } else {
                error!("No connection to database , could not get group info");
                "".to_string()
            }
        }
        DbStore::Remote(_) => todo!(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_media_sources(handle: AppHandle, hash: String) -> Vec<MediaSource> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                get_media_sources_impl(&hash, pool).await
            } else {
                error!("No connection to database , could not get media source");
                vec![]
            }
        }
        DbStore::Remote(remote_store) => {
            remote_store.client.get_media_sources(&hash).await.unwrap()
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn set_media_favorite(handle: AppHandle, hash: String, state: bool) {
    let app_state = handle.state::<DatabaseState>();
    let connection_state = app_state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                set_media_favorite_impl(&hash, state, pool).await;
            } else {
                error!("No connection to database , could not get media source");
            }
        }
        DbStore::Remote(remote_store) => remote_store
            .client
            .set_media_favorite(&hash, state)
            .await
            .unwrap(),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn get_video_length(handle: AppHandle, hash: String) -> Option<f64> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                kasa_core::media::get_video_length_impl(&hash, pool).await
            } else {
                error!("No connection to database, could not get video length");
                None
            }
        }
        DbStore::Remote(remote_store) => remote_store.client.get_video_length(&hash).await.unwrap(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_top_n_closest_for_media(
    handle: AppHandle,
    hash: String,
    n: i64,
) -> Vec<EmbeddingDistance> {
    let state = handle.state::<DatabaseState>();
    let connection_state = state.0.lock().await;

    match &*connection_state {
        DbStore::Local(db_store) => {
            let connection_guard = db_store.db.lock().await.clone();
            if let Some(pool) = connection_guard.as_ref() {
                let closest = get_top_n_closest_for_media_impl(pool, &hash, n).await;
                match closest {
                    Ok(c) => return c,
                    Err(e) => {
                        error!("{}", e.to_string());
                        return vec![];
                    }
                }
            } else {
                return vec![];
            }
        }
        DbStore::Remote(remote_store) => remote_store
            .client
            .get_top_n_closest_for_media(&hash, n)
            .await
            .unwrap(),
    }
}
