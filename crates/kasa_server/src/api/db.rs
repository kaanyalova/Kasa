use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use kasa_core::db::{
    TagQueryOutput,
    db_info::{ThumbsDBInfo, get_thumbs_db_info_impl},
    query_tags_impl,
};
use serde::Deserialize;
use sqlx::{Pool, Sqlite};
use utoipa::IntoParams;

use crate::api::Databases;

#[derive(Deserialize, IntoParams)]
pub struct TagQueryParams {
    query: String,
    limit: i64,
}

#[utoipa::path(
    get,
    path = "/query_tags",
    params(TagQueryParams),
    responses(
        (status = 200, description = "Query Successful", body = Vec<TagQueryOutput>),
    ),
)]
pub async fn query_tags(
    State(dbs): State<Databases>,
    Query(params): Query<TagQueryParams>,
) -> (StatusCode, Json<Vec<TagQueryOutput>>) {
    (
        StatusCode::OK,
        Json(query_tags_impl(params.query, params.limit, &dbs.db).await),
    )
}
