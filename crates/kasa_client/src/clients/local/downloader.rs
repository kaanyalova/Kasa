use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;

use anyhow::Result;
use kasa_core::{
    config::global_config::GlobalConfig,
    downloaders::download_queue::{
        DownloadJob, Downloader, DownloaderContext, init_extractors,
    },
};
use kasa_python::GalleryDlStatus;
use kasa_python::extractors::TagExtractor;
use log::error;
use sqlx::{Pool, Sqlite};
use tokio::sync::mpsc;

use crate::clients::EventCallback;
use crate::clients::downloader::KasaDownloader;
use crate::events::{DownloaderProgressUpdatedEvent, KasaEvent, TagsUpdatedEvent};

pub struct LocalDownloader {
    pub tx: mpsc::Sender<DownloadJob>,
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
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

        let statuses_on_progress = statuses.clone();
        let event_on_progress = on_event.clone();
        let on_progress = move |status: &GalleryDlStatus| {
            statuses_on_progress
                .lock()
                .unwrap()
                .insert(status.url_hash.clone(), status.clone());

            if let Some(on_event) = &event_on_progress {
                on_event(KasaEvent::DownloaderProgressUpdated(
                    DownloaderProgressUpdatedEvent {},
                ));
            }
        };

        let statuses_on_done = statuses.clone();
        let event_on_done = on_event.clone();
        let on_done = move |hash: String| {
            statuses_on_done.lock().unwrap().remove(&hash);

            if let Some(on_event) = &event_on_done {
                on_event(KasaEvent::DownloaderProgressUpdated(
                    DownloaderProgressUpdatedEvent {},
                ));
                on_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));
            }
        };

        let (mut downloader, tx) =
            Downloader::init(db, thumbs_db, config.clone(), on_progress, on_done);

        let tagger_worker = downloader.workers.tagger_worker.clone();
        let statuses_run = statuses.clone();
        let extractors = init_extractors(tagger_worker, config)?;
        downloader.set_extractors(extractors.clone());

        tokio::spawn(async move {
            let ctx = DownloaderContext::from_values(statuses_run);
            downloader.run(&ctx).await;
        });

        Ok(Self {
            tx,
            statuses,
            extractors,
        })
    }

    pub async fn queue_download(&self, url: &str) {
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
        for extractor in self.extractors.iter() {
            if let Err(e) = extractor.reload() {
                error!("failed to reload extractor: {e}");
            }
        }
    }
}
