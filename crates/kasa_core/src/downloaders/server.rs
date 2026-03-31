use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use axum_macros::debug_handler;
use kasa_python::init_interpreter;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/*
use super::gallery_dl::PyTrustMe;
async fn start_dl_extension_server() {
    let app = Router::new().route(
        "check_status",
        post(download_from_url).with_state(Arc::new(Mutex::new(PyTrustMe(init_interpreter())))),
    );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}




#[derive(Serialize, Deserialize, Debug)]
struct DownloadPayload {
    url: String,
}

#[debug_handler]
async fn download_from_url(
    State(py_state): State<Arc<Mutex<PyTrustMe>>>,
    Json(payload): Json<DownloadPayload>,
) -> StatusCode {

    let intr =&py_state.lock().await.0;

    download_and_index_impl(intr, payload.url, "todo", pool, pool_thumbs, when_done)
    todo!()
}
*/
