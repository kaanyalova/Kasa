use std::path::PathBuf;

use kasa_core::config::global_config::{
    GlobalConfig, get_config_impl, set_db_path_impl, set_thumbs_db_path_impl, set_value_resolution,
    set_value_str,
};

#[tauri::command(async)]
#[specta::specta]
pub fn get_config() -> GlobalConfig {
    get_config_impl()
}

#[tauri::command(async)]
#[specta::specta]
pub fn set_config_value(category: &str, key: &str, valu: &str) {
    set_value_str(category, key, valu)
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
