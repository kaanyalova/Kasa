use axum::{Json, extract::State, http::StatusCode};
use kasa_core::{db::schema::Media, tags::search::SearchCriteria};

use crate::api::Databases;

#[utoipa::path(
    post,
    path = "/search",
    request_body = SearchCriteria,
    responses(
        (status = 200, description = "Search Successful", body = Vec<Media>),
    ),
)]
pub async fn search(
    State(dbs): State<Databases>,
    Json(criteria): Json<SearchCriteria>,
) -> (StatusCode, Json<Vec<Media>>) {
    let mut query = criteria.to_query();
    let media: Vec<Media> = query.build_query_as().fetch_all(&dbs.db).await.unwrap();

    (StatusCode::OK, Json(media))
}
