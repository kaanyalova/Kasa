use serde::{Deserialize, Serialize};

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

#[derive(specta::Type, Serialize, Deserialize)]
pub struct RawImage {
    width: u32,
    height: u32,
    bytes: Vec<u8>,
}

#[tauri::command(async)]
#[specta::specta]
pub fn image_path_to_rgba_bytes(path: &str) -> RawImage {
    let img = image::open(path).unwrap().to_rgba8();

    RawImage {
        width: img.width(),
        height: img.height(),
        bytes: img.into_raw(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub fn open_with_system_default_app(path: &str) {
    opener::open(path).unwrap();
}
