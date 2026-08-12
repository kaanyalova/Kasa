use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct DownloaderProgressUpdatedEvent {}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct TagsUpdatedEvent {}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct CacheUpdatedEvent {
    pub reload_virtual_list: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct DbsUpdatedEvent {
    /// was the db just created
    pub new_db: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct MediaServerDownEvent {}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]

pub struct OpenMediaModalEvent {
    pub hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event)]
pub struct CloseMediaModalEvent {}
