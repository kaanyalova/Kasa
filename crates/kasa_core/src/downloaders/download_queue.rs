use anyhow::Result;
use kasa_python::GalleryDlStatus;
use kasa_python::extractors::TagExtractor;
use kasa_python::extractors::configurable::ConfigurableExtractor;
use kasa_python::extractors::scriptable::PythonTagExtractor;
use kasa_python::worker::downloader::GalleryDlDownloadWorker;
use kasa_python::worker::tagger::TaggerWorker;
use log::{error, info};
use serde::Deserialize;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::config::global_config::GlobalConfig;
use crate::config::global_config::get_tag_extractors_dir;
use crate::downloaders::gallery_dl::{download_and_index_impl, hash_url};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DownloaderStateUpdate {
    OnProgress(GalleryDlStatus),
    OnDone(String),
    OnError { url_hash: String, error: String },
}

#[derive(Serialize, Deserialize)]
pub struct DownloadJob {
    pub url: String,
}

#[derive(Clone)]
pub struct Extractors(Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>);

impl Extractors {
    pub fn reload(&self) {
        for extractor in self.0.iter() {
            if let Err(e) = extractor.reload() {
                error!("failed to reload extractor: {e}");
            }
        }
    }
}

pub struct DownloaderWorker {
    rx: mpsc::Receiver<DownloadJob>,
    db: Pool<Sqlite>,
    thumbs_db: Pool<Sqlite>,
    config: GlobalConfig,
    extractors: Extractors,
}

impl DownloaderWorker {
    pub fn init(
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        config: GlobalConfig,
    ) -> Result<(Self, mpsc::Sender<DownloadJob>)> {
        let (tx, rx) = mpsc::channel::<DownloadJob>(32);

        let extractors = init_extractors()?;

        Ok((
            Self {
                rx,
                db: db.clone(),
                thumbs_db: thumbs_db.clone(),
                config,
                extractors,
            },
            tx,
        ))
    }

    pub async fn run<F>(&mut self, on_update: F)
    where
        F: Fn(DownloaderStateUpdate) + Send + Sync + Clone + 'static,
    {
        let on_update_progress = on_update.clone();
        let on_progress = move |status: &GalleryDlStatus| {
            on_update_progress(DownloaderStateUpdate::OnProgress(status.clone()));
        };

        let downloader_worker = Arc::new(GalleryDlDownloadWorker::new(on_progress).unwrap());

        info!("python runtimes initialized, waiting for download jobs...");
        while let Some(job) = self.rx.recv().await {
            self.process_download_job(&downloader_worker, on_update.clone(), job)
                .await;
        }
        info!("download loop done?");
    }

    async fn process_download_job<F>(
        &mut self,
        downloader_worker: &Arc<GalleryDlDownloadWorker>,
        on_update: F,
        job: DownloadJob,
    ) where
        F: Fn(DownloaderStateUpdate) + Send + Sync + Clone + 'static,
    {
        let on_update_error = on_update.clone();
        let on_done = move |hash: String| on_update(DownloaderStateUpdate::OnDone(hash));
        let interpreter = downloader_worker.clone();
        let db = self.db.clone();
        let thumbs_db = self.thumbs_db.clone();
        let output_path = self.config.downloader.output_path.clone();
        let extractors = self.extractors.clone();

        let extractor_refs: Vec<&(dyn TagExtractor + Send + Sync)> =
            extractors.0.iter().map(|e| e.as_ref()).collect();

        if let Err(e) = download_and_index_impl(
            interpreter,
            &job.url,
            &output_path,
            &db,
            &thumbs_db,
            on_done,
            &extractor_refs,
        )
        .await
        {
            on_update_error(DownloaderStateUpdate::OnError {
                url_hash: hash_url(&job.url),
                error: e.to_string(),
            });
        }
    }

    pub fn extractors(&self) -> Extractors {
        self.extractors.clone()
    }
}

pub fn init_extractors() -> Result<Extractors> {
    let extractors_dir = get_tag_extractors_dir()?;

    // should i get rid of this oop shit?
    let worker = Arc::new(TaggerWorker::new()?);
    let python_extractor = PythonTagExtractor::init(worker, &extractors_dir)?;

    let configurable_exactor = ConfigurableExtractor::init(&extractors_dir)?;

    Ok(Extractors(Arc::new(vec![
        Box::new(python_extractor),
        Box::new(configurable_exactor),
    ])))
}
