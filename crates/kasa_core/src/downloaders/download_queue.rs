use anyhow::Result;
use kasa_python::GalleryDlStatus;
use kasa_python::PyTrustMe;
use kasa_python::extractors::TagExtractor;
use kasa_python::extractors::configurable::ConfigurableExtractor;
use kasa_python::extractors::scriptable::PythonTagExtractor;
use kasa_python::init_interpreter;
use kasa_python::init_interpreter_with_gallery_dl;
use log::info;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Pool;
use sqlx::Sqlite;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::mpsc;

use crate::config::global_config::GlobalConfig;
use crate::config::global_config::get_tag_extractors_dir;
use crate::downloaders::gallery_dl::download_and_index_impl;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DownloaderStateUpdate {
    OnProgress(GalleryDlStatus),
    OnDone(String),
}

pub struct PythonInterpreters {
    // not thread safe, obviously
    pub downloader_interpreter: Arc<PyTrustMe>,
    // locking this with a mutex is fine?, tag extraction shouldn't take that long
    // i might implement a proper queue though
    pub tagger_interpreter: Arc<Mutex<PyTrustMe>>,
}

#[derive(Serialize, Deserialize)]
pub struct DownloadJob {
    pub url: String,
}

// this is the stuff thats on the "client" side
// what should tauri implement for the server/local modes,
// things that are updated by ws etc.
pub struct DownloaderContext {
    statuses: Arc<Mutex<HashMap<String, GalleryDlStatus>>>,
}

impl DownloaderContext {
    pub fn from_values(statuses: Arc<Mutex<HashMap<String, GalleryDlStatus>>>) -> Self {
        Self { statuses }
    }

    pub fn new() -> Self {
        Self {
            statuses: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

// this is the stuff for the "server" side
pub struct Downloader<FP: Fn(&GalleryDlStatus) + Send + Sync, FD: Fn(String) + Send + Sync> {
    pub interpreters: PythonInterpreters,
    rx: mpsc::Receiver<DownloadJob>,
    // url hashes mapped to the current download statuses
    db: Pool<Sqlite>,
    thumbs_db: Pool<Sqlite>,
    config: GlobalConfig,
    on_downloader_progress: FP,
    on_downloader_done: FD,
    extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
}

impl<FP, FD> Downloader<FP, FD>
where
    FP: Fn(&GalleryDlStatus) + Send + Sync + Clone + 'static,
    FD: Fn(String) + Send + Sync + Clone + 'static,
{
    pub fn init(
        db: Pool<Sqlite>,
        thumbs_db: Pool<Sqlite>,
        config: GlobalConfig,
        on_progress: FP,
        on_done: FD,
    ) -> (Self, mpsc::Sender<DownloadJob>) {
        let (tx, rx) = mpsc::channel::<DownloadJob>(32);

        let interpreters = PythonInterpreters {
            downloader_interpreter: Arc::new(PyTrustMe(init_interpreter_with_gallery_dl())),
            tagger_interpreter: Arc::new(Mutex::new(PyTrustMe(init_interpreter()))),
        };

        let extractors = Arc::new(vec![]);

        (
            Self {
                interpreters,
                rx,
                on_downloader_done: on_done,
                on_downloader_progress: on_progress,
                db: db.clone(),
                thumbs_db: thumbs_db.clone(),
                config,
                extractors,
            },
            tx,
        )
    }

    pub async fn run(&mut self, ctx: &DownloaderContext) {
        info!("python runtimes initialized, waiting for download jobs...");
        while let Some(job) = self.rx.recv().await {
            self.process_download_job(ctx, job).await;
        }
        info!("download loop done?");
    }

    async fn process_download_job(&mut self, ctx: &DownloaderContext, job: DownloadJob) {
        let dummy_status = GalleryDlStatus::new_placeholder(&job.url);
        {
            let mut locked = ctx.statuses.lock().unwrap();
            locked.insert(dummy_status.url_hash.clone(), dummy_status.clone());
        }
        (self.on_downloader_progress)(&dummy_status);

        let statuses_progress = ctx.statuses.clone();
        let statuses_done = ctx.statuses.clone();
        let on_progress_cb = self.on_downloader_progress.clone();
        let on_done_cb = self.on_downloader_done.clone();
        let interpreter = self.interpreters.downloader_interpreter.clone();
        let db = self.db.clone();
        let thumbs_db = self.thumbs_db.clone();
        let output_path = self.config.downloader.output_path.clone();
        let extractors = self.extractors.clone();

        let on_progress = move |status: &GalleryDlStatus| {
            (on_progress_cb)(status);
            statuses_progress
                .lock()
                .unwrap()
                .insert(status.url_hash.clone(), status.clone());
        };

        let on_done = move |hash: String| {
            statuses_done.lock().unwrap().remove(&hash);
            (on_done_cb)(hash);
        };

        let extractor_refs: Vec<&(dyn TagExtractor + Send + Sync)> =
            extractors.iter().map(|e| e.as_ref()).collect();

        let _download = download_and_index_impl(
            interpreter,
            &job.url,
            &output_path,
            &db,
            &thumbs_db,
            on_done,
            on_progress,
            &extractor_refs,
        )
        .await;
    }

    pub fn set_extractors(&mut self, extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>) {
        self.extractors = extractors;
    }
}

pub fn init_extractors(
    interpreter: Arc<Mutex<PyTrustMe>>,
    config: &GlobalConfig,
) -> Result<Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>> {
    let extractors_dir = get_tag_extractors_dir()?;
    let python_extractor = PythonTagExtractor::init(interpreter.clone(), &extractors_dir)?;

    let configurable_exactor = ConfigurableExtractor::init(&extractors_dir)?;

    Ok(Arc::new(vec![
        Box::new(python_extractor),
        Box::new(configurable_exactor),
    ]))
}
