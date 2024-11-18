use kasa_core::linux::get_desktop::get_desktop_impl;

#[tauri::command(async)]
#[specta::specta]
pub fn get_desktop() -> Option<String> {
    get_desktop_impl()
}
