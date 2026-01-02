use std::collections::HashMap;
use std::sync::Arc;

use kasa_core::config::global_config::get_configurable_tag_extractor_path;
use kasa_core::downloaders::gallery_dl::{PyTrustMe, get_download_progress_impl};
use kasa_core::{
    config::global_config::get_config_impl, downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::extractors::configurable::{ExtractorConfig, get_extractors_from_path};
use kasa_python::{GalleryDlStatuses, init_interpreter};
use log::{error, trace};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

use crate::db::DbStore;

#[derive(Default)]
pub struct PythonStore {
    interpreter: Mutex<Option<Arc<PyTrustMe>>>,
}

async fn init_or_get_python(handle: &AppHandle) -> Arc<PyTrustMe> {
    // this gets the interpreter without holding the mutex, i dont need a mutex but
    // tauri forces me to have one
    let python_state = handle.state::<PythonStore>();
    let mut guard = python_state.interpreter.lock().await;

    if let Some(py) = guard.as_ref() {
        return py.clone();
    } else {
        let new_interpreter = Arc::new(PyTrustMe(init_interpreter()));
        *guard = Some(new_interpreter.clone());

        new_interpreter
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn download_and_index(handle: AppHandle, url: String) {
    let connection_state = handle.state::<DbStore>();
    let python_state = handle.state::<PythonStore>();
    let tag_extractor_state = handle.state::<ExtractorsStore>();
    let connection_guard = connection_state.db.lock().await;
    let connection_guard_thumbs = connection_state.thumbs_db.lock().await;

    let mut conection_guard_extractors = tag_extractor_state.extractors.lock().await;

    if let (Some(db), Some(thumbs_db)) =
        (connection_guard.as_ref(), connection_guard_thumbs.as_ref())
    {
        let py = init_or_get_python(&handle).await;

        let cfg = get_config_impl();
        if let Some(extractors) = conection_guard_extractors.as_mut() {
            download_and_index_impl(
                &py,
                &url,
                &cfg.downloader.output_path,
                db,
                thumbs_db,
                &|| handle.emit("media_updated", "").unwrap(),
                extractors,
            )
            .await
            .unwrap();
        } else {
            let extractors_path = get_configurable_tag_extractor_path().unwrap();
            let extractors = get_extractors_from_path(&extractors_path.to_string_lossy()).unwrap();

            download_and_index_impl(
                &py,
                &url,
                &cfg.downloader.output_path,
                db,
                thumbs_db,
                &|| handle.emit("media_updated", "").unwrap(),
                &extractors,
            )
            .await
            .unwrap();

            trace!("Replacing extractors state start...");
            *conection_guard_extractors = Some(extractors);
            trace!("Replacing extractors state complete");
        }
    }
}

#[tauri::command(async)]
#[specta::specta]
pub async fn get_download_progress(handle: AppHandle) -> GalleryDlStatuses {
    let python_state = handle.state::<PythonStore>();
    let connection_guard_py = python_state.interpreter.lock().await;

    if let Some(py) = connection_guard_py.as_ref() {
        let status = get_download_progress_impl(py).await.unwrap();
        return status;
    } else {
        error!(
            "cannot get python interpreter for looking up download statuses returning empty hashmap"
        );
        return GalleryDlStatuses::default();
    }
}

#[derive(Debug, Default)]
pub struct ExtractorsStore {
    extractors: Mutex<Option<HashMap<String, ExtractorConfig>>>,
}
