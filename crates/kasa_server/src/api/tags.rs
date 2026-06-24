use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use kasa_core::tags::{
    AllTagsOrderingCriteria, TagWithCount, get_list_of_all_tags_with_details_impl,
    get_tags_as_text_impl, remove_tags, update_tags_impl,
};
use serde::Deserialize;
use tracing_subscriber::registry::Data;
use utoipa::IntoParams;

use crate::api::Databases;

#[derive(Deserialize, IntoParams)]
pub struct UpdateTagsParams {
    pub raw_input: String,
    pub hash: String,
}

#[utoipa::path(
    put,
    path = "/update_tags",
    params(UpdateTagsParams),
    responses(
        (status = 200, description = "Update Successful"),
    ),
)]
pub async fn update_tags(
    State(dbs): State<Databases>,
    Query(params): Query<UpdateTagsParams>,
) -> StatusCode {
    update_tags_impl(&params.raw_input, params.hash, &dbs.db).await;

    StatusCode::OK
}

#[derive(Deserialize, IntoParams)]
pub struct DeleteTagsParams {
    pub hash: String,
    pub tags: Vec<String>,
}

#[
utoipa::path(
    delete,
    path = "/delete_tags",
    params(DeleteTagsParams),
    responses(
        (status = 200, description = "Delete Successful"),
    ),
)]
pub async fn delete_tags(
    State(dbs): State<Databases>,
    Query(params): Query<DeleteTagsParams>,
) -> StatusCode {
    remove_tags(params.tags, &dbs.db, Some(params.hash)).await;
    StatusCode::OK
}

#[derive(Deserialize, IntoParams)]
pub struct GetTagsAsTextParams {
    pub hash: String,
}

#[utoipa::path(
    get,
    path = "/get_tags_as_text",
    params(GetTagsAsTextParams),
    responses(
        (status = 200, description = "Get Tags Successful", body = String),
    ),
)]
pub async fn get_tags_as_text(
    State(dbs): State<Databases>,
    Query(params): Query<GetTagsAsTextParams>,
) -> (StatusCode, Json<Option<String>>) {
    let text = get_tags_as_text_impl(&params.hash, &dbs.db).await;
    (StatusCode::OK, Json(Some(text)))
}

#[derive(Deserialize, IntoParams)]
pub struct GetListOfAllTagsWithDetailsParams {
    ordering_criteria: AllTagsOrderingCriteria,
}

#[
utoipa::path(
    get,
    path = "/get_list_of_all_tags_with_details",
    params(GetListOfAllTagsWithDetailsParams),
    responses(
        (status = 200, description = "Get List of All Tags Successful", body = Vec<TagWithCount>),
    ),
)
]
pub async fn get_list_of_all_tags_with_details(
    State(dbs): State<Databases>,
    Query(params): Query<GetListOfAllTagsWithDetailsParams>,
) -> (StatusCode, Json<Vec<TagWithCount>>) {
    (
        StatusCode::OK,
        Json(get_list_of_all_tags_with_details_impl(&dbs.db, params.ordering_criteria).await),
    )
}
