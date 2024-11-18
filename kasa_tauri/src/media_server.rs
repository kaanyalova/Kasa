use axum::Router;
use kasa_core::db::schema::Media;
use log::{debug, trace};
use specta::specta;
use sqlx::query_scalar;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{
    oneshot::{self, Receiver, Sender},
    Mutex,
};
use tower_http::services::ServeFile;

use crate::db::DbStore;

#[derive(Debug, Default)]
pub struct MediaServerStore {
    serve_path: Mutex<Option<String>>,
    ptr: Mutex<i64>,
}

#[tauri::command(async)]
/// Returns the pointer to close the server
#[specta::specta]
pub async fn serve_media(handle: AppHandle, hash: String) {
    let state = handle.state::<MediaServerStore>();

    let db_state = handle.state::<DbStore>();
    let db_guard = db_state.db.lock().await;

    let Some(pool) = db_guard.as_ref() else {
        return;
    };

    let path: String = query_scalar("SELECT path FROM Path WHERE hash = ? LIMIT 1")
        .bind(&hash)
        .fetch_one(pool)
        .await
        .unwrap();

    // return if we are tying to serve the same file
    if state.serve_path.lock().await.as_ref() == Some(&path) {
        return;
    }

    if state.serve_path.lock().await.is_some() {
        // Safety: should be only dereferenced when there is an active server
        let kill = unsafe {
            let kill_ptr = state.ptr.lock().await.clone() as *mut Receiver<()>;
            let kill_box = Box::from_raw(kill_ptr);
            *kill_box
        };

        tokio::spawn(async move {
            drop(kill);
        });
    }

    let (mut kill_rx, kill_tx): (Sender<()>, Receiver<()>) = oneshot::channel();

    let boxed = Box::new(kill_tx);

    let raw = Box::into_raw(boxed) as i64;

    *state.ptr.lock().await = raw;
    *state.serve_path.lock().await = Some(path.clone());

    let handle = handle.clone();
    tokio::spawn(async move {
        let serve_dir = ServeFile::new(&path);
        let router = Router::new().nest_service("/", serve_dir);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:3169")
            .await
            .unwrap();
        trace!("serving! path: {}", &path);
        axum::serve(listener, router)
            .with_graceful_shutdown(async move { kill_rx.closed().await })
            .await
            .unwrap();

        handle.emit("media_server_down", ()).unwrap();
    });
}

/// This should be only called once from js side
#[tauri::command(async)]
#[specta::specta]
pub async fn close_server(handle: AppHandle) {
    let state = handle.state::<MediaServerStore>();

    if state.serve_path.lock().await.is_some() {
        // Safety: should be only dereferenced when there is an active server
        let kill = unsafe {
            let kill_ptr = state.ptr.lock().await.clone() as *mut Receiver<()>;
            let kill_box = Box::from_raw(kill_ptr);
            *kill_box
        };

        *state.serve_path.lock().await = None;

        tokio::spawn(async move {
            drop(kill);
        });
    }
}
