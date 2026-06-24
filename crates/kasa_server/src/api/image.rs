use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
};
use kasa_core::thumbnail::thumbnailer::generate_or_get_thumbnail_from_db_impl;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::Databases;

#[derive(Deserialize, IntoParams)]
pub struct GetThumbnailParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_thumbnail",
    params(GetThumbnailParams),
    responses(
        (status = 200, description = "thumbnail image", content_type = "image/png", body = [u8]),
    ),
)]
pub async fn get_thumbnail(
    State(dbs): State<Databases>,
    Query(params): Query<GetThumbnailParams>,
) -> (StatusCode, HeaderMap, Vec<u8>) {
    //let thumbnail = generate_or_get_thumbnail_from_db_impl(hash, &dbs.db, &dbs.thumbs_db);
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "image/png".parse().unwrap());

    let thumbnail =
        generate_or_get_thumbnail_from_db_impl(&params.hash, &dbs.db, &dbs.thumbs_db).await;

    (StatusCode::OK, headers, thumbnail)
}
