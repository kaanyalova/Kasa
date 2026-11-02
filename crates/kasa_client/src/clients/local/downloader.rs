use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;

use anyhow::Result;
use kasa_core::{
    config::global_config::GlobalConfig,
    downloaders::download_queue::{
        DownloadJob, DownloaderStateUpdate, DownloaderWorker, Extractors,
    },
};
use kasa_python::GalleryDlStatus;
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc;

use crate::clients::EventCallback;
use crate::clients::downloader::KasaDownloader;
use crate::events::{DownloaderProgressUpdatedEvent, KasaEvent, TagsUpdatedEvent};

#[derive(Clone)]
pub struct LocalDownloader {
    pub tx: mpsc::Sender<DownloadJob>,
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub extractors: Extractors,
    on_event: Option<EventCallback>,
}

impl KasaDownloader for LocalDownloader {}

impl LocalDownloader {
    pub async fn new(
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        config: &GlobalConfig,
        on_event: Option<EventCallback>,
    ) -> Result<Self> {
        let statuses = Arc::new(SyncMutex::new(HashMap::new()));

        let statuses_on_update = statuses.clone();
        let event_on_update = on_event.clone();
        let on_update = move |update: DownloaderStateUpdate| match update {
            DownloaderStateUpdate::OnProgress(status) => {
                statuses_on_update
                    .lock()
                    .unwrap()
                    .insert(status.url_hash.clone(), status);

                if let Some(on_event) = &event_on_update {
                    on_event(KasaEvent::DownloaderProgressUpdated(
                        DownloaderProgressUpdatedEvent {},
                    ));
                }
            }
            DownloaderStateUpdate::OnDone(hash) => {
                statuses_on_update.lock().unwrap().remove(&hash);

                if let Some(on_event) = &event_on_update {
                    on_event(KasaEvent::DownloaderProgressUpdated(
                        DownloaderProgressUpdatedEvent {},
                    ));
                    on_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));
                }
            }
            DownloaderStateUpdate::OnError { url_hash, .. } => {
                statuses_on_update.lock().unwrap().remove(&url_hash);

                if let Some(on_event) = &event_on_update {
                    on_event(KasaEvent::DownloaderProgressUpdated(
                        DownloaderProgressUpdatedEvent {},
                    ));
                }
            }
        };

        let (mut downloader, tx) = DownloaderWorker::init(db, thumbs_db, config.clone())?;

        let extractors = downloader.extractors();

        tokio::spawn(async move {
            downloader.run(on_update).await;
        });

        Ok(Self {
            tx,
            statuses,
            extractors,
            on_event,
        })
    }

    pub async fn queue_download(&self, url: &str) {
        // register the placeholder right away, otherwise queued jobs are invisible until
        // the downloader actually starts working on them
        let placeholder = GalleryDlStatus::new_placeholder(url, "queued");
        self.statuses
            .lock()
            .unwrap()
            .entry(placeholder.url_hash.clone())
            .or_insert(placeholder);

        if let Some(on_event) = &self.on_event {
            on_event(KasaEvent::DownloaderProgressUpdated(
                DownloaderProgressUpdatedEvent {},
            ));
        }

        self.tx
            .send(DownloadJob {
                url: url.to_string(),
            })
            .await
            .unwrap();
    }

    pub fn get_statuses(&self) -> HashMap<String, GalleryDlStatus> {
        self.statuses.lock().unwrap().clone()
    }

    pub fn reload_extractors(&self) {
        self.extractors.reload();
    }
}
