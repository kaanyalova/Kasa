use anyhow::{Ok, Result};
use kasa_core::config::global_config::{GlobalConfig, get_tag_extractors_dir};
use kasa_core::downloaders::download_queue::{
    DownloadJob, Downloader, DownloaderContext, DownloaderStateUpdate, init_extractors,
};
use kasa_core::{
    config::global_config::get_config_impl, downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::extractors::TagExtractor;
use kasa_python::{GalleryDlStatus, init_interpreter, init_interpreter_with_gallery_dl};
use log::{error, info};
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
        DownloaderStore::Local(local) => local
            .tx
            .send(DownloadJob { url: url.clone() })
            .await
            .unwrap(),
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

pub struct LocalDownloaderStore {
    pub tx: mpsc::Sender<DownloadJob>,
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
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

impl DownloaderStore {
    pub async fn new_local(
        handle: AppHandle,
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        config: &GlobalConfig,
    ) -> Result<Self> {
        let statuses = Arc::new(SyncMutex::new(HashMap::new()));

        let statuses_on_progress = statuses.clone();
        let handle_on_progress = handle.clone();
        let on_progress = move |status: &GalleryDlStatus| {
            let mut locked = statuses_on_progress.lock().unwrap();
            locked.insert(status.url_hash.clone(), status.clone());

            DownloaderProgressUpdatedEvent {}
                .emit(&handle_on_progress)
                .unwrap();
        };

        let statuses_on_done = statuses.clone();
        let handle_on_done = handle.clone();
        let on_done = move |hash: String| {
            let mut locked = statuses_on_done.lock().unwrap();
            locked.remove(&hash);

            DownloaderProgressUpdatedEvent {}
                .emit(&handle_on_done)
                .unwrap();

            TagsUpdatedEvent {}.emit(&handle_on_done).unwrap();

            let handle_search = handle_on_done.clone();
            tokio::spawn(async move {
                search(handle_search, false).await;
            });
        };

        let (mut downloader, tx) =
            Downloader::init(db, thumbs_db, config.clone(), on_progress, on_done);

        let tagger_worker = downloader.workers.tagger_worker.clone();
        let statuses_run = statuses.clone();

        let extractors = init_extractors(tagger_worker, config)?;

        let extractors_clone = extractors.clone();
        downloader.set_extractors(extractors_clone);

        tokio::spawn(async move {
            let ctx = DownloaderContext::from_values(statuses_run);
            downloader.run(&ctx).await;
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
                    search(handle_search, false).await;
                });
            }
        };

        client.listen_for_downloader_updates(on_update).await?;

        let remote_store = Self::Remote(RemoteDownloaderStore { statuses, client });
        Ok(remote_store)
    }
}
