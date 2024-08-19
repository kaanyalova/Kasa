#[tauri::command]
#[specta::specta]
/// gets the provided env var, returns an empty string if the env var doesn't exist or something goes wrong
pub fn get_env_var(_var: &str) -> String {
    let var = std::env::var_os(_var);
    match var {
        Some(var) => var.to_string_lossy().to_string(),
        None => "".to_string(),
    }
}
