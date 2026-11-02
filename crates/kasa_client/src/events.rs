use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, uniffi::Enum)]
pub enum KasaEvent {
    DownloaderProgressUpdated(DownloaderProgressUpdatedEvent),
    TagsUpdated(TagsUpdatedEvent),
    CacheUpdated(CacheUpdatedEvent),
    MediaServerDown(MediaServerDownEvent),
    OpenMediaModal(OpenMediaModalEvent),
    CloseMediaModal(CloseMediaModalEvent),
    DatabaseConnection(DatabaseConnectionEvent),
}
#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct DownloaderProgressUpdatedEvent {}

#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct TagsUpdatedEvent {}

#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct CacheUpdatedEvent {
    pub reload_virtual_list: bool,
}

#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct MediaServerDownEvent {}

#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]

pub struct OpenMediaModalEvent {
    pub hash: String,
}

#[derive(
    uniffi::Record, Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event,
)]
pub struct CloseMediaModalEvent {}

#[derive(Clone, Debug, Serialize, Deserialize, specta::Type, tauri_specta::Event, uniffi::Enum)]
#[serde(tag = "type", content = "data")]
pub enum DatabaseConnectionEvent {
    RemoteConnected,
    LocalConnected,
    Uninitialize,
    Failed(String),
}
