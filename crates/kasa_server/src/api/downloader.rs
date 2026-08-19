use std::net::SocketAddr;

use axum::{
    Json,
    body::Bytes,
    extract::{
        ConnectInfo, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::StatusCode,
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use kasa_core::downloaders::download_queue::{DownloadJob, DownloaderStateUpdate};
use kasa_python::GalleryDlStatus;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast::error::RecvError;
use tokio::sync::{broadcast, mpsc};
use tracing::log::warn;
use tracing::{error, trace};
use utoipa::IntoParams;

#[derive(Clone, Debug)]
pub struct DownloaderState {
    pub job_tx: mpsc::Sender<DownloadJob>,
    pub update_broadcast: broadcast::Sender<DownloaderStateUpdate>,
}

#[utoipa::path(
    get,
    path = "/listen_for_download_updates",
    responses(
        (status = 101, description = "websocket upgrade")
    )
)]
pub async fn listen_for_download_updates(
    State(downloader_state): State<DownloaderState>,
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    trace!("{} is requesting ws upgrade for download updates", addr);
    ws.on_upgrade(move |socket| {
        handle_download_updates(socket, addr, downloader_state.update_broadcast.subscribe())
    })
}

async fn handle_download_updates(
    mut socket: WebSocket,
    who: SocketAddr,
    mut events: broadcast::Receiver<DownloaderStateUpdate>,
) {
    trace!("{} connected to ws for download updates", who);

    let (mut sender, mut receiver) = socket.split();

    let send_whom = who.clone();
    let mut send_task = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(Duration::from_secs(10));
        loop {
            tokio::select! {
                event = events.recv() => {
                    match event {
                        Ok(event) => {
                            let serialize_result = serde_json::to_string(&event);

                            match serialize_result {
                                Ok(t) => {
                                    // println!("sending msg {:?} to {:?}", &event, &send_whom);
                                    trace!("sending msg {:?} to {:?}", &event, &send_whom);
                                    sender.send(Message::text(t)).await.unwrap();
                                }
                                Err(e) => {
                                    error!("Failed to serialize event for {:?}: {:?}", &send_whom, e);
                                }
                            }
                        }
                        Err(RecvError::Lagged(l)) => {trace!("skipped {} ws messages because of lag", l)},
                        Err(RecvError::Closed) => break,
                    }
                }
                _ = heartbeat.tick() => {
                    trace!("sending heartbeat to {:?}", &send_whom);
                    if sender
                        .send(Message::text(r#"{"type":"heartbeat"}"#))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            match msg {
                Ok(Message::Close(_)) | Err(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => {
            trace!("Send task completed for {:?}", &who);
            recv_task.abort();
        }
        _ = (&mut recv_task) => {
            trace!("Receive task completed for {:?}", &who);
            send_task.abort();
        }
    }
}

#[derive(IntoParams, Deserialize, Serialize)]
pub struct PushDownloadRequest {
    pub url: String,
}

#[utoipa::path(
    post,
    path = "/push_download",
    params(PushDownloadRequest),
    responses(
        (status = 200, description = "added download job successfully"),)
    )]
pub async fn push_download(
    State(downloader_state): State<DownloaderState>,
    Json(request): Json<PushDownloadRequest>,
) -> StatusCode {
    trace!("pushing download with url: {}", request.url);
    let job = DownloadJob { url: request.url };
    downloader_state.job_tx.send(job).await.unwrap();
    StatusCode::OK
}
