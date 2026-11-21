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

pub enum DownloaderJob {
    Download(DownloadJob),
    ReloadExtractors,
}

pub fn job_channel() -> (mpsc::Sender<DownloaderJob>, mpsc::Receiver<DownloaderJob>) {
    mpsc::channel(32)
}

pub async fn run<F>(
    db: Pool<Sqlite>,
    thumbs_db: Pool<Sqlite>,
    config: GlobalConfig,
    mut rx: mpsc::Receiver<DownloaderJob>,
    on_update: F,
) -> Result<()>
where
    F: Fn(DownloaderStateUpdate) + Send + Sync + Clone + 'static,
{
    let extractors = init_extractors()?;

    let on_update_progress = on_update.clone();
    let on_progress = move |status: &GalleryDlStatus| {
        on_update_progress(DownloaderStateUpdate::OnProgress(status.clone()));
    };

    let downloader_worker = Arc::new(GalleryDlDownloadWorker::new(on_progress).unwrap());

    info!("python runtimes initialized, waiting for download jobs...");
    while let Some(job) = rx.recv().await {
        match job {
            DownloaderJob::Download(job) => {
                process_download_job(
                    &downloader_worker,
                    &db,
                    &thumbs_db,
                    &config,
                    &extractors,
                    &on_update,
                    job,
                )
                .await;
            }
            DownloaderJob::ReloadExtractors => {
                for extractor in extractors.iter() {
                    if let Err(e) = extractor.reload() {
                        error!("failed to reload extractor: {e}");
                    }
                }
            }
        }
    }
    info!("download loop done?");
    Ok(())
}

async fn process_download_job<F>(
    downloader_worker: &Arc<GalleryDlDownloadWorker>,
    db: &Pool<Sqlite>,
    thumbs_db: &Pool<Sqlite>,
    config: &GlobalConfig,
    extractors: &Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
    on_update: &F,
    job: DownloadJob,
) where
    F: Fn(DownloaderStateUpdate) + Send + Sync + Clone + 'static,
{
    let on_update_error = on_update.clone();
    let on_done = {
        let on_update = on_update.clone();
        move |hash: String| on_update(DownloaderStateUpdate::OnDone(hash))
    };

    let output_path = config.downloader.output_path.clone();

    if let Err(e) = download_and_index_impl(
        downloader_worker.clone(),
        &job.url,
        &output_path,
        db,
        thumbs_db,
        on_done,
        extractors.clone(),
    )
    .await
    {
        on_update_error(DownloaderStateUpdate::OnError {
            url_hash: hash_url(&job.url),
            error: e.to_string(),
        });
    }
}

pub fn init_extractors() -> Result<Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>> {
    let extractors_dir = get_tag_extractors_dir()?;

    // should i get rid of this oop shit?
    let worker = Arc::new(TaggerWorker::new()?);
    let python_extractor = PythonTagExtractor::init(worker, &extractors_dir)?;

    let configurable_exactor = ConfigurableExtractor::init(&extractors_dir)?;

    Ok(Arc::new(vec![
        Box::new(python_extractor),
        Box::new(configurable_exactor),
    ]))
}
