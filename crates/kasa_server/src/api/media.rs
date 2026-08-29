use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use kasa_core::{
    db::{
        embeddings::{EmbeddingDistance, get_top_n_closest_for_media_impl},
        schema::MediaSource,
    },
    media::{
        MediaInfo, SourceCategoryGroupedTags, TagWithDetails, get_info_impl, get_media_name_impl,
        get_media_sources_impl, get_media_type_impl, get_tags_detailed_impl,
        get_tags_grouped_by_source_categories_impl, get_valid_path_impl, get_video_length_impl,
        set_media_favorite_impl,
    },
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::api::Databases;

#[derive(Deserialize, IntoParams)]
pub struct GetMediaInfoParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_info",
    params(GetMediaInfoParams),
    responses(
        (status = 200, description = "Query Successful", body = Option<MediaInfo>),
    ),
)]
pub async fn get_info(
    State(dbs): State<Databases>,
    Query(params): Query<GetMediaInfoParams>,
) -> (StatusCode, Json<Option<MediaInfo>>) {
    (
        StatusCode::OK,
        Json(Some(get_info_impl(&params.hash, &dbs.db).await)),
    )
}

#[derive(Deserialize, IntoParams)]
pub struct GetTagsParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_tags",
    params(GetTagsParams),
    responses(
        (status = 200, description = "Query Successful", body = Vec<TagWithDetails>),
    ),
)]
pub async fn get_tags(
    State(dbs): State<Databases>,
    Query(params): Query<GetTagsParams>,
) -> (StatusCode, Json<Vec<TagWithDetails>>) {
    (
        StatusCode::OK,
        Json(get_tags_detailed_impl(&params.hash, &dbs.db).await),
    )
}

#[derive(Deserialize, IntoParams)]
pub struct GetMediaTypeParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_media_type",
    params(GetMediaTypeParams),
    responses(
        (status = 200, description = "Query Successful", body = String),
    ),
)]
pub async fn get_media_type(
    State(dbs): State<Databases>,
    Query(params): Query<GetMediaTypeParams>,
) -> (StatusCode, Json<String>) {
    (
        StatusCode::OK,
        Json(get_media_type_impl(&params.hash, &dbs.db).await),
    )
}

#[derive(Deserialize, IntoParams)]
pub struct GetTagsGroupedBySourceCategoriesParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_tags_grouped_by_source_categories",
    params(GetTagsGroupedBySourceCategoriesParams),
    responses(
        (status = 200, description = "Query Successful", body = SourceCategoryGroupedTags),
    ),
)]
pub async fn get_tags_grouped_by_source_categories(
    State(dbs): State<Databases>,
    Query(params): Query<GetTagsGroupedBySourceCategoriesParams>,
) -> (StatusCode, Json<SourceCategoryGroupedTags>) {
    (
        StatusCode::OK,
        Json(get_tags_grouped_by_source_categories_impl(&params.hash, &dbs.db).await),
    )
}

#[derive(Deserialize, IntoParams)]
pub struct GetMediaNameParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_media_name",
    params(GetMediaNameParams),
    responses(
        (status = 200, description = "Query Successful", body = String),
    ),
)]
pub async fn get_media_name(
    State(dbs): State<Databases>,
    Query(params): Query<GetMediaNameParams>,
) -> (StatusCode, String) {
    (
        StatusCode::OK,
        get_media_name_impl(&params.hash, &dbs.db).await,
    )
}

#[derive(Deserialize, IntoParams)]
pub struct GetMediaSourcesParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_media_sources",
    params(GetMediaSourcesParams),
    responses(
        (status = 200, description = "Query Successful", body = Vec<MediaSource>),
    ),
)]
pub async fn get_media_sources(
    State(dbs): State<Databases>,
    Query(params): Query<GetMediaSourcesParams>,
) -> (StatusCode, Json<Vec<MediaSource>>) {
    (
        StatusCode::OK,
        Json(get_media_sources_impl(&params.hash, &dbs.db).await),
    )
}

#[derive(Deserialize, IntoParams)]
pub struct SetFavoriteParams {
    hash: String,
    is_favorite: bool,
}

#[utoipa::path(
    put,
    path = "/set_media_favorite",
    params(SetFavoriteParams),
    responses(
        (status = 200, description = "Query Successful"),
    ),
)]
pub async fn set_media_favorite(
    State(dbs): State<Databases>,
    Query(params): Query<SetFavoriteParams>,
) -> StatusCode {
    set_media_favorite_impl(&params.hash, params.is_favorite, &dbs.db).await;

    StatusCode::OK
}

#[derive(Deserialize, IntoParams)]
pub struct GetVideoLengthParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_video_length",
    params(GetVideoLengthParams),
    responses(
        (status = 200, description = "Query Successful", body = Option<f64>),
    ),
)]
pub async fn get_video_length(
    State(dbs): State<Databases>,
    Query(params): Query<GetVideoLengthParams>,
) -> (StatusCode, Json<Option<f64>>) {
    let length = get_video_length_impl(&params.hash, &dbs.db).await;

    (StatusCode::OK, Json(length))
}

#[derive(Deserialize, IntoParams)]
pub struct GetTopNClosestForMediaParams {
    hash: String,
    n: i64,
}

#[utoipa::path(
    get,
    path = "/get_top_n_closest_for_media",
    params(GetTopNClosestForMediaParams),
    responses(
        (status = 200, description = "Query Successful", body = Vec<EmbeddingDistance>),
    ),
)]
pub async fn get_top_n_closest_for_media(
    State(dbs): State<Databases>,
    Query(params): Query<GetTopNClosestForMediaParams>,
) -> (StatusCode, Json<Vec<EmbeddingDistance>>) {
    let distances = get_top_n_closest_for_media_impl(&dbs.db, &params.hash, params.n)
        .await
        .unwrap();

    (StatusCode::OK, Json(distances))
}

#[derive(Deserialize, IntoParams)]

pub struct GetValidPathParams {
    hash: String,
}

#[utoipa::path(
    get,
    path = "/get_valid_path",
    params(GetValidPathParams),
    responses(
        (status = 200, description = "Query Successful", body = String),
    ),
)]
pub async fn get_valid_path(
    State(dbs): State<Databases>,
    Query(params): Query<GetValidPathParams>,
) -> (StatusCode, String) {
    let path = get_valid_path_impl(&params.hash, &dbs.db).await;

    (StatusCode::OK, path)
}
