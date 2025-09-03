use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use net_result::response_data;

pub async fn get_artifact(
    State(state): State<ClientState>,
    Path((asset_id, track_id)): Path<(i64, i32)>,
) -> ClientResult<impl IntoResponse> {
    let result = state
        .artifact_repo
        .find_by_obj_track(asset_id, track_id)
        .await?;

    let Some(artifact) = result else {
        return Err(ClientError::NotFound);
    };

    Ok(response_data(artifact))
}
