
use kasa_core::config::global_config::{get_config_impl, GlobalConfig};

#[tauri::command]
#[specta::specta]
pub fn get_config() -> GlobalConfig {
    get_config_impl()
}
