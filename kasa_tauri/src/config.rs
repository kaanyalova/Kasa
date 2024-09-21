use kasa_core::config::global_config::{
    get_config_impl, set_value_resolution, set_value_str, GlobalConfig,
};

#[tauri::command]
#[specta::specta]
pub fn get_config() -> GlobalConfig {
    get_config_impl()
}

#[tauri::command]
#[specta::specta]
pub fn set_config_value(category: &str, key: &str, valu: &str) {
    set_value_str(category, key, valu)
}

#[tauri::command]
#[specta::specta]
pub fn set_config_resolution_value(height: u32, width: u32) {
    set_value_resolution(height, width)
}
