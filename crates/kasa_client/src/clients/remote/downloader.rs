use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as SyncMutex;

use anyhow::Result;
use kasa_core::{
    config::global_config::GlobalConfig,
    downloaders::download_queue::DownloaderStateUpdate,
};
use kasa_python::GalleryDlStatus;

use crate::clients::EventCallback;
use crate::clients::downloader::KasaDownloader;
use crate::clients::remote::rest_client::RemoteDownloaderClient;
use crate::events::{DownloaderProgressUpdatedEvent, KasaEvent, TagsUpdatedEvent};

pub struct RemoteDownloader {
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    pub client: RemoteDownloaderClient,
}

impl KasaDownloader for RemoteDownloader {}

impl RemoteDownloader {
    pub async fn new(config: &GlobalConfig, on_event: Option<EventCallback>) -> Result<Self> {
        let mut client = RemoteDownloaderClient::new(&config.db.db_path);

        let statuses = Arc::new(SyncMutex::new(HashMap::new()));

        let statuses_update = statuses.clone();
        client
            .listen_for_downloader_updates(move |update: DownloaderStateUpdate| {
                match update {
                    DownloaderStateUpdate::OnProgress(gallery_dl_status) => {
                        statuses_update
                            .lock()
                            .unwrap()
                            .insert(gallery_dl_status.url_hash.clone(), gallery_dl_status);

                        if let Some(on_event) = &on_event {
                            on_event(KasaEvent::DownloaderProgressUpdated(
                                DownloaderProgressUpdatedEvent {},
                            ));
                        }
                    }
                    DownloaderStateUpdate::OnDone(hash) => {
                        statuses_update.lock().unwrap().remove(&hash);

                        if let Some(on_event) = &on_event {
                            on_event(KasaEvent::DownloaderProgressUpdated(
                                DownloaderProgressUpdatedEvent {},
                            ));
                            on_event(KasaEvent::TagsUpdated(TagsUpdatedEvent {}));
                        }
                    }
                }
            })
            .await?;

        Ok(Self { statuses, client })
    }

    pub async fn push_download(&self, url: &str) -> Result<()> {
        self.client.push_download(url).await
    }

    pub fn get_statuses(&self) -> HashMap<String, GalleryDlStatus> {
        self.statuses.lock().unwrap().clone()
    }
}
