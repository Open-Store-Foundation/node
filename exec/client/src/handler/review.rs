use crate::env::default_page_size;
use crate::data::models::NewReview;
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use net_result::{response_data, response_null};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ReviewListParams {
    #[serde(default = "default_page_size")]
    pub size: i64,
    #[serde(default)]
    pub offset: i64,
}

pub async fn get_reviews_for_object(
    State(state): State<ClientState>,
    Path(asset_id): Path<i64>,
    Query(params): Query<ReviewListParams>,
) -> ClientResult<impl IntoResponse> {
    if params.size > 100 {
        return Err(
            ClientError::BadInput(
                "query parameter 'size' max 100".to_string(),
            )
        );
    }

    state.object_repo.find_by_id(asset_id)
        .await?;

    let reviews = state
        .review_repo
        .find_by_asset_id(asset_id, params.size, params.offset)
        .await?;

    Ok(response_data(reviews))
}

pub async fn create_review(
    State(state): State<ClientState>,
    Json(payload): Json<NewReview>,
) -> ClientResult<impl IntoResponse> {
    if !(1..=5).contains(&payload.rating) {
        return Err(ClientError::BadInput("Rating must be between 1 and 5".to_string()));
    }

    if payload.user_id.is_empty() {
        return Err(ClientError::BadInput("User ID cannot be empty".to_string()));
    }
    
    state.object_repo.find_by_id(payload.asset_id)
        .await?;

    state.review_repo.create(payload)
        .await?; 

    Ok((StatusCode::CREATED, response_null::<String>()))
}