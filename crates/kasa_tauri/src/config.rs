use std::path::PathBuf;

use crate::AppHandle;
use kasa_core::config::{
    extractor_scripts::{
        create_or_get_extractor_contents_impl, create_or_get_path_for_extractor_impl,
        get_example_metadata_for_extractor_impl, get_existing_extractor_names_impl,
    },
    global_config::{
        GlobalConfig, get_config_impl, set_db_path_impl, set_thumbs_db_path_impl, set_value,
        set_value_resolution,
    },
};
use tauri::Manager;

use crate::db::DbStore;

#[tauri::command(async)]
#[specta::specta]
pub fn get_config() -> GlobalConfig {
    get_config_impl()
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_config_value_f64(category: &str, key: &str, valu: f64) {
    set_value(category, key, valu);
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_config_value_bool(category: &str, key: &str, valu: bool) {
    set_value(category, key, valu);
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_config_value_str(category: &str, key: &str, valu: &str) {
    set_value(category, key, valu);
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_config_resolution_value(height: u32, width: u32) {
    set_value_resolution(height, width)
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_db_path(path: &str) {
    set_db_path_impl(&PathBuf::from(path));
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_thumbs_db_path(path: &str) {
    set_thumbs_db_path_impl(&PathBuf::from(path));
}

#[tauri::command(async)]
#[specta::specta]
pub fn create_or_get_path_for_extractor(extractor_name: &str, file_extension: &str) -> String {
    create_or_get_path_for_extractor_impl(extractor_name, file_extension)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string()
}

#[tauri::command(async)]
#[specta::specta]
pub fn create_or_get_extractor_contents(extractor_name: &str, file_extension: &str) -> String {
    create_or_get_extractor_contents_impl(extractor_name, file_extension).unwrap()
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_existing_extractor_names(handle: AppHandle) -> Vec<String> {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        get_existing_extractor_names_impl(pool).await.unwrap()
    } else {
        vec![]
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_example_metadata_for_extractor(handle: AppHandle, name: String) -> String {
    let connection_state = handle.state::<DbStore>();
    let connection_guard = connection_state.db.lock().await.clone();

    if let Some(pool) = connection_guard.as_ref() {
        get_example_metadata_for_extractor_impl(pool, &name)
            .await
            .unwrap()
    } else {
        let response =
            r#"{"error" : "Could not query the database for an example, is the db connected?"}"#;
        response.to_owned()
    }
}
