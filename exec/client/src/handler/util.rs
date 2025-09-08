use axum::extract::State;
use crate::result::ClientResult;
use axum::response::IntoResponse;
use net_result::response_ok;
use crate::state::ClientState;

pub async  fn handle_health(
    State(_): State<ClientState>,
) -> ClientResult<impl IntoResponse> {
    return Ok(response_ok())
}
