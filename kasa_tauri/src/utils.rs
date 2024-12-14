use std::path::Path;

use tauri::path;

#[tauri::command(async)]
#[specta::specta]
/// gets the provided env var, returns an empty string if the env var doesn't exist or something goes wrong
pub fn get_env_var(envvar: &str) -> String {
    let var = std::env::var_os(envvar);
    match var {
        Some(var) => var.to_string_lossy().to_string(),
        None => "".to_string(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn image_path_to_rgba_bytes(path: &str) -> Vec<u8> {
    let img = image::open(path).unwrap().to_rgba8();
    img.into_raw()
}

#[tauri::command(async)]
#[specta::specta]
pub fn open_with_system_default_app(path: &str) {
    opener::open(path).unwrap();
}
