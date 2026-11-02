use anyhow::{Ok, Result};
use kasa_core::config::global_config::{GlobalConfig, get_tag_extractors_dir};
use kasa_core::downloaders::download_queue::{
    DownloadJob, DownloaderStateUpdate, DownloaderWorker, Extractors,
};
use kasa_core::{
    config::global_config::get_config_impl, downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::{GalleryDlStatus, init_interpreter, init_interpreter_with_gallery_dl};
use log::info;
use sqlx::{Pool, Sqlite};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_specta::Event;
use tokio::sync::{Mutex, mpsc};

use crate::db::{DatabaseState, DbStore};
use crate::events::{DownloaderProgressUpdatedEvent, TagsUpdatedEvent};
use crate::remote_client::{RemoteClient, RemoteDownloaderClient};
use crate::search::search;
use std::sync::Mutex as SyncMutex;

#[tauri::command(async)]
#[specta::specta]
pub async fn queue_download_job(handle: AppHandle, url: String) {
    let state = handle.state::<DownloaderState>();
    let store = state.0.lock().await;

    let store_type = match &*store {
        DownloaderStore::Local(_) => "local",
        DownloaderStore::Remote(_) => "remote",
        DownloaderStore::Uninitialized => "uninitialized",
    };

    match &*store {
        DownloaderStore::Local(local) => {
            let placeholder = GalleryDlStatus::new_placeholder(&url, "queued");
            local
                .statuses
                .lock()
                .unwrap()
                .entry(placeholder.url_hash.clone())
                .or_insert(placeholder);

            DownloaderProgressUpdatedEvent {}.emit(&handle).unwrap();

            local
                .tx
                .send(DownloadJob { url: url.clone() })
                .await
                .unwrap()
        }
        DownloaderStore::Remote(remote) => remote.client.push_download(&url).await.unwrap(),

        DownloaderStore::Uninitialized => {}
    }

    info!(
        "queued download job with context: {}, url: {}",
        store_type, url
    );
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_downloader_statuses(handle: AppHandle) -> HashMap<String, GalleryDlStatus> {
    let state = handle.state::<DownloaderState>();
    let store = state.0.lock().await;

    match &*store {
        DownloaderStore::Local(local) => {
            let locked = local.statuses.lock().unwrap();
            locked.clone()
        }
        DownloaderStore::Remote(remote) => {
            let locked = remote.statuses.lock().unwrap();
            locked.clone()
        }
        DownloaderStore::Uninitialized => HashMap::new(),
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn reload_extractors(handle: AppHandle) {
    let state = handle.state::<DownloaderState>();
    let store = state.0.lock().await;

    match &*store {
        DownloaderStore::Local(local) => local.extractors.reload(),
        DownloaderStore::Remote(_) => {}
        _ => {}
    }
}

pub struct LocalDownloaderStore {
    pub tx: mpsc::Sender<DownloadJob>,
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub extractors: Extractors,
}

pub struct RemoteDownloaderStore {
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub client: RemoteDownloaderClient,
}

pub enum DownloaderStore {
    Local(LocalDownloaderStore),
    Remote(RemoteDownloaderStore),
    Uninitialized,
}

pub struct DownloaderState(pub Mutex<DownloaderStore>);

impl DownloaderState {
    pub fn uninitialized() -> Self {
        Self(Mutex::new(DownloaderStore::Uninitialized))
    }
}

impl DownloaderStore {
    pub async fn new_local(
        handle: AppHandle,
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        config: &GlobalConfig,
    ) -> Result<Self> {
        let statuses = Arc::new(SyncMutex::new(HashMap::new()));

        let statuses_on_update = statuses.clone();
        let handle_on_update = handle.clone();
        let on_update = move |update: DownloaderStateUpdate| match update {
            DownloaderStateUpdate::OnProgress(status) => {
                statuses_on_update
                    .lock()
                    .unwrap()
                    .insert(status.url_hash.clone(), status);

                DownloaderProgressUpdatedEvent {}
                    .emit(&handle_on_update)
                    .unwrap();
            }
            DownloaderStateUpdate::OnDone(hash) => {
                statuses_on_update.lock().unwrap().remove(&hash);

                DownloaderProgressUpdatedEvent {}
                    .emit(&handle_on_update)
                    .unwrap();

                TagsUpdatedEvent {}.emit(&handle_on_update).unwrap();

                let handle_search = handle_on_update.clone();
                tokio::spawn(async move {
                    search(handle_search).await;
                });
            }
            DownloaderStateUpdate::OnError { url_hash, .. } => {
                statuses_on_update.lock().unwrap().remove(&url_hash);

                DownloaderProgressUpdatedEvent {}
                    .emit(&handle_on_update)
                    .unwrap();
            }
        };

        let (mut downloader, tx) = DownloaderWorker::init(db, thumbs_db, config.clone())?;

        let extractors = downloader.extractors();

        tokio::spawn(async move {
            downloader.run(on_update).await;
        });

        Ok(Self::Local(LocalDownloaderStore {
            tx,
            statuses,
            extractors,
        }))
    }

    pub async fn new_remote(handle: AppHandle, config: &GlobalConfig) -> Result<Self> {
        let mut client = RemoteDownloaderClient::new(&config.db.db_path.clone());

        let statuses = Arc::new(SyncMutex::new(HashMap::new()));

        let statuses_update_clone = statuses.clone();
        let on_update = move |u: DownloaderStateUpdate| match u {
            DownloaderStateUpdate::OnProgress(gallery_dl_status) => {
                let mut locked = statuses_update_clone.lock().unwrap();
                locked.insert(gallery_dl_status.url_hash.clone(), gallery_dl_status);
                DownloaderProgressUpdatedEvent {}.emit(&handle).unwrap();
            }
            DownloaderStateUpdate::OnDone(hash) => {
                let mut locked = statuses_update_clone.lock().unwrap();
                locked.remove(&hash);

                DownloaderProgressUpdatedEvent {}.emit(&handle).unwrap();

                TagsUpdatedEvent {}.emit(&handle).unwrap();

                let handle_search = handle.clone();
                tokio::spawn(async move {
                    search(handle_search).await;
                });
            }
            DownloaderStateUpdate::OnError { url_hash, .. } => {
                let mut locked = statuses_update_clone.lock().unwrap();
                locked.remove(&url_hash);

                DownloaderProgressUpdatedEvent {}.emit(&handle).unwrap();
            }
        };

        client.listen_for_downloader_updates(on_update).await?;

        let remote_store = Self::Remote(RemoteDownloaderStore { statuses, client });
        Ok(remote_store)
    }
}
