use crate::env::default_page_size;
use crate::data::id::{ObjTypeId, PlatformId};
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use net_result::{response_data, response_empty};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchParams {
    pub content: String,
    #[serde(rename = "platform")]
    pub platform_id: PlatformId,
    #[serde(rename = "type")]
    pub type_id: Option<ObjTypeId>,
    #[serde(rename = "category_id")]
    pub category_id: Option<i32>,
    #[serde(default = "default_page_size")]
    pub size: i64,
    #[serde(default)]
    pub offset: i64,
}

pub async fn search_objects(
    State(state): State<ClientState>,
    Query(params): Query<SearchParams>,
) -> ClientResult<impl IntoResponse> {
    if params.size > 100 {
        return Err(
            ClientError::BadInput(
                "query parameter 'size' max 100".to_string(),
            )
        );
    }

    let search_term = params.content;
    if search_term.len() < 3 {
        return Err(
            ClientError::BadInput(
                "query parameter 'content' at least 3 symbols".to_string(),
            )
        );
    }

    let results = match params.category_id {
        Some(value) => state
            .search_repo
            .search_by_category(
                &search_term,
                params.platform_id.into(),
                params.type_id,
                value,
                params.size,
                params.offset,
            )
            .await?,
        None => state
            .search_repo
            .search(
                &search_term,
                params.platform_id.into(),
                params.type_id,
                params.size,
                params.offset,
            )
            .await?
    };

    Ok(response_data(results))
}
