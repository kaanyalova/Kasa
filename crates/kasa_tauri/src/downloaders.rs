use std::collections::HashMap;
use std::sync::Arc;

use kasa_core::config::global_config::get_tag_extractors_dir;
use kasa_core::{
    config::global_config::get_config_impl, downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::extractors::TagExtractor;
use kasa_python::extractors::configurable::ConfigurableExtractor;
use kasa_python::extractors::scriptable::PythonTagExtractor;
use kasa_python::{GalleryDlStatus, PyTrustMe, init_interpreter, init_interpreter_with_gallery_dl};
use log::error;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::mpsc;

use crate::db::{DatabaseState, DbStore};
use std::sync::Mutex as SyncMutex;

pub struct PythonStore {
    // not thread safe, obviously
    pub downloader_interpreter: Arc<PyTrustMe>,
    // locking this with a mutex is fine?, tag extraction shouldn't take that long
    // i might implement a proper queue though
    pub tagger_interpreter: Arc<SyncMutex<PyTrustMe>>,
}

impl PythonStore {
    pub fn init_interpreter() -> Self {
        Self {
            downloader_interpreter: Arc::new(PyTrustMe(init_interpreter_with_gallery_dl())),
            tagger_interpreter: Arc::new(SyncMutex::new(PyTrustMe(init_interpreter()))),
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn queue_download_job(handle: AppHandle, url: String) {
    let state = handle.state::<DownloaderStore>();
    state.channel.send(DownloadJob { url }).await.unwrap();
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_downloader_statuses(handle: AppHandle) -> HashMap<String, GalleryDlStatus> {
    let statuses = handle.state::<DownloaderStore>();
    let locked = statuses.statuses.lock().unwrap();
    locked.clone()
}

pub struct DownloadJob {
    pub url: String,
}

pub struct DownloaderStore {
    pub channel: mpsc::Sender<DownloadJob>,
    pub statuses: Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
}

impl DownloaderStore {
    pub fn init_queue(handle: AppHandle) -> Self {
        dbg!("Init downloader queue");

        let (tx, mut rx) = tokio::sync::mpsc::channel::<DownloadJob>(32);

        let statuses = Arc::new(SyncMutex::new(HashMap::new()));
        let statuses_cloned = statuses.clone();

        tauri::async_runtime::spawn(async move {
            while let Some(job) = rx.recv().await {
                let cfg = get_config_impl();

                let db_handle = handle.state::<DatabaseState>();
                let connection_state = db_handle.0.lock().await;
                let python_state = handle.state::<PythonStore>();
                let tag_extractor_state = handle.state::<ExtractorsStore>();

                match &*connection_state {
                    DbStore::Local(db_store) => {
                        let connection_guard = db_store.db.lock().await.clone();
                        let connection_guard_thumbs = db_store.thumbs_db.lock().await.clone();

                        let extractors_cloned = tag_extractor_state.extractors.clone();

                        process_download_job(
                            &handle,
                            &statuses_cloned,
                            &cfg,
                            &python_state,
                            &connection_guard,
                            &connection_guard_thumbs,
                            extractors_cloned,
                            job,
                        )
                        .await;
                    }
                    DbStore::Remote(_) => todo!(),
                }
            }
        });

        Self {
            channel: tx,
            statuses: statuses.clone(),
        }
    }
}

async fn process_download_job(
    handle: &AppHandle,
    statuses: &Arc<SyncMutex<HashMap<String, GalleryDlStatus>>>,
    cfg: &kasa_core::config::global_config::GlobalConfig,
    python_state: &tauri::State<'_, PythonStore>,
    connection_guard: &Option<sqlx::Pool<sqlx::Sqlite>>,
    connection_guard_thumbs: &Option<sqlx::Pool<sqlx::Sqlite>>,
    extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
    job: DownloadJob,
) {
    // create the dummy status until the progress actually updates it
    {
        let dummy_status = GalleryDlStatus::new_placeholder(&job.url);
        let mut locked = statuses.lock().unwrap();
        locked.insert(dummy_status.url_hash.clone(), dummy_status);
        handle.emit("downloader_progress_updated", "").unwrap();
    }

    let handle_cloned_when_done = handle.clone();
    let handle_cloned_on_progress = handle.clone();

    let statuses_cloned_when_done = statuses.clone();
    let statuses_cloned_on_progress = statuses.clone();

    let on_download_done = |url_hash: String| {
        handle_cloned_when_done.emit("media_updated", "").unwrap();
        handle_cloned_when_done.emit("tags_updated", "").unwrap();

        let mut locked = statuses_cloned_when_done.lock().unwrap();
        locked.remove(&url_hash);
        handle_cloned_when_done
            .emit("downloader_progress_updated", "")
            .unwrap();
    };

    let on_progress = move |status: GalleryDlStatus| {
        let mut locked = statuses_cloned_on_progress.lock().unwrap();
        locked.insert(status.url_hash.clone(), status);
        handle_cloned_on_progress
            .emit("downloader_progress_updated", "")
            .unwrap();
    };

    if let (Some(db), Some(thumbs_db)) =
        (connection_guard.as_ref(), connection_guard_thumbs.as_ref())
    {
        dbg!("downloading url:{}", &job.url);

        let extractors_refs: Vec<&(dyn TagExtractor + Send + Sync)> = extractors
            .iter()
            .map(|e| e.as_ref() as &(dyn TagExtractor + Send + Sync))
            .collect();

        let download_status = download_and_index_impl(
            python_state.downloader_interpreter.clone(),
            &job.url,
            &cfg.downloader.output_path,
            db,
            thumbs_db,
            on_download_done,
            on_progress,
            &extractors_refs,
        )
        .await;

        match download_status {
            Ok(_) => {}
            Err(e) => {
                let e = e.to_string();
                error!("{}", e);
            }
        }
    } else {
        error!("databases are not found for the downloader");
    }
}

pub struct ExtractorsStore {
    extractors: Arc<Vec<Box<dyn TagExtractor + Send + Sync>>>,
}

impl ExtractorsStore {
    pub fn init(handle: AppHandle) -> Self {
        let extractors_path = get_tag_extractors_dir().unwrap();

        let interpreter = &handle.state::<PythonStore>().tagger_interpreter;
        let python_extractor =
            PythonTagExtractor::init(interpreter.clone(), &extractors_path).unwrap();

        let configurable_extractor = ConfigurableExtractor::init(&extractors_path).unwrap();
        //let extractors = get_extractors_from_path(&extractors_path.to_string_lossy()).unwrap();

        Self {
            extractors: Arc::new(vec![
                Box::new(python_extractor),
                Box::new(configurable_extractor),
            ]),
        }
    }
}
