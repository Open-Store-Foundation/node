use crate::data::models::{Category, NewCategory, Asset};
use crate::data::id::ObjTypeId;
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::State, Json};
use net_result::{response_null, response_nullable};
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Deserialize)]
pub struct CategoryParams {
    #[serde(rename = "type")]
    pub type_id: ObjTypeId,
}

// For POST /admin/set_categories
#[derive(Deserialize, Debug)]
pub struct SetCategoriesRequest {
    pub categories: Vec<NewCategory>,
}

pub async fn set_categories(
    State(state): State<ClientState>,
    Query(params): Query<CategoryParams>,
    Json(payload): Json<SetCategoriesRequest>,
) -> ClientResult<impl IntoResponse> {
    info!(
        "Admin: Setting categories for type {:?}: {:?}",
        params.type_id, // Log the param even if unused by current logic
        payload.categories
    );

    // let mut created_categories = Vec::new();
    // for new_cat in payload.categories {
    //     created_categories.push(new_cat);
    // }

    Ok((StatusCode::CREATED, response_nullable()))
}