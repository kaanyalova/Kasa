use std::collections::HashMap;
use std::{borrow::BorrowMut, sync::Arc};

use kasa_core::config::global_config::get_configurable_tag_extractor_path;
use kasa_core::downloaders::gallery_dl::PyTrustMe;
use kasa_core::{
    config::global_config::get_config_impl, downloaders::gallery_dl::download_and_index_impl,
};
use kasa_python::extractors::configurable::{get_extractors_from_path, ExtractorConfig};
use kasa_python::{gdl_download, init_interpreter};
use log::trace;
use rustpython_vm::Interpreter;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_fs::init;
use tokio::sync::Mutex;

use crate::db::DbStore;

#[derive(Default)]
pub struct PythonStore {
    interpreter: Mutex<Option<PyTrustMe>>,
}

fn init_or_get_python() {}

#[tauri::command(async)]
#[specta::specta]
pub async fn download_and_index(handle: AppHandle, url: String) {
    let connection_state = handle.state::<DbStore>();
    let python_state = handle.state::<PythonStore>();
    let tag_extractor_state = handle.state::<ExtractorsStore>();
    let connection_guard = connection_state.db.lock().await;
    let connection_guard_thumbs = connection_state.thumbs_db.lock().await;

    //if connection_guard_py.as_ref().is_some() {
    //} else {
    //
    //}

    *python_state.interpreter.lock().await = Some(PyTrustMe(init_interpreter()));

    let connection_guard_py = python_state.interpreter.lock().await;
    let mut conection_guard_extractors = tag_extractor_state.extractors.lock().await;

    if let (Some(db), Some(thumbs_db), Some(py)) = (
        connection_guard.as_ref(),
        connection_guard_thumbs.as_ref(),
        connection_guard_py.as_ref(),
    ) {
        let cfg = get_config_impl();
        if let Some(extractors) = conection_guard_extractors.as_mut() {
            download_and_index_impl(
                py,
                &url,
                &cfg.downloader.output_path,
                db,
                thumbs_db,
                &|| handle.emit("media_updated", "").unwrap(),
                &extractors,
            )
            .await
            .unwrap();
        } else {
            let extractors_path = get_configurable_tag_extractor_path().unwrap();
            let extractors = get_extractors_from_path(&extractors_path.to_string_lossy()).unwrap();

            download_and_index_impl(
                py,
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

#[derive(Debug, Default)]
pub struct ExtractorsStore {
    extractors: Mutex<Option<HashMap<String, ExtractorConfig>>>,
}
