use alloy::hex::ToHexExt;
use alloy::primitives::Address;
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use tracing::{debug, info};
use codegen_contracts::ext::ToChecksum;
use net_result::response_data;
use crate::data::dto::AndroidPublishingResponse;

pub async fn get_object_by_id(
    State(state): State<ClientState>,
    Path(asset_id): Path<i64>,
) -> ClientResult<impl IntoResponse> {
    let object = state
        .object_repo
        .find_by_id(asset_id)
        .await?;

    let Some(obj) = object else {
        return Err(ClientError::NotFound);
    };

    Ok(response_data(obj))
}

pub async fn get_object_by_address(
    State(state): State<ClientState>,
    Path(address): Path<Address>,
) -> ClientResult<impl IntoResponse> {
    let addr = address.lower_checksum();
    let object = state.object_repo
        .find_by_address(addr.as_str())
        .await?;

    let Some(obj) = object else {
        return Err(ClientError::NotFound);
    };

    Ok(response_data(obj))
}

pub async fn get_object_status_by_address(
    State(state): State<ClientState>,
    Path(address): Path<Address>,
) -> ClientResult<impl IntoResponse> {
    let published = state.publishing_repo
        .get_publishing_by_address(&address)
        .await?;

    let reviewing = state
        .validation_repo.get_requests_by_address(address)
        .await?;

    let response = AndroidPublishingResponse {
        published, reviewing
    };

    Ok(response_data(response))
}