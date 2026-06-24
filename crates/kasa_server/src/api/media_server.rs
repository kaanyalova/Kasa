use axum::{
    extract::{Query, Request, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite, query_scalar};
use std::path::Path;
use tower::ServiceExt;
use tower_http::services::ServeFile;
use tracing::trace;
use utoipa::IntoParams;

use crate::api::Databases;

pub async fn get_first_valid_file_by_hash(hash: &str, pool: &Pool<Sqlite>) -> Option<String> {
    let paths: Vec<String> = query_scalar("SELECT path FROM Path WHERE hash = ?")
        .bind(hash)
        .fetch_all(pool)
        .await
        .ok()?;

    trace!("Found possible paths for hash {}: {:?}", hash, paths);

    paths.into_iter().find(|p| Path::new(p).exists())
}

#[derive(Deserialize, IntoParams)]
pub struct ServeMediaParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/media",
    params(ServeMediaParams),
    responses(
        (status = 200, description = "Media file stream"),
        (status = 404, description = "File not found"),
    ),
)]
pub async fn serve_media(
    State(dbs): State<Databases>,
    Query(params): Query<ServeMediaParams>,
    request: Request,
) -> impl IntoResponse {
    match get_first_valid_file_by_hash(&params.hash, &dbs.db).await {
        Some(path) => {
            trace!("Serving media file: {}", path);
            let service = ServeFile::new(&path);
            match service.oneshot(request).await {
                Ok(response) => response.into_response(),
                Err(_) => {
                    (StatusCode::INTERNAL_SERVER_ERROR, "failed to serve file").into_response()
                }
            }
        }
        None => (StatusCode::NOT_FOUND, "no file found for this hash").into_response(),
    }
}
