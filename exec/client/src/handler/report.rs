use crate::data::models::NewReport;
use crate::result::{ClientError, ClientResult};
use crate::state::ClientState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use net_result::response_nullable;

// TODO add filter
// TRAFFICKING_NARCOTICS(1),
// TRAFFICKING_COUNTERFEIT(2),
// TRAFFICKING_STOLEN_GOODS(3),
// CYBERCRIME_MALWARE(4),
// CYBERCRIME_HACKING(5),
// CYBERCRIME_CARDING(6),
// FINANCIAL_CRIME_LAUNDERING(7),
// FINANCIAL_CRIME_FRAUD(8),
// FINANCIAL_CRIME_STOLEN_DATA(9),
// EXPLOITATION_CSAM(10),
// EXPLOITATION_HUMAN_TRAFFICKING(11),
// EXPLOITATION_VIOLENCE_FOR_HIRE(12),
// EXPLOITATION_TERRORISM(13),
// PRIVACY_DOXING(14),
// PRIVACY_SURVEILLANCE(15);
pub async fn create_report(
    State(state): State<ClientState>,
    Json(payload): Json<NewReport>,
) -> ClientResult<impl IntoResponse> {
    if payload.email.is_empty() || !payload.email.contains('@') { // TODO check email
        return Err(ClientError::BadInput("A valid email address is required".to_string()));
    }

    if payload.category_id <= 0 {
        return Err(ClientError::BadInput("A valid reason ID is required".to_string()));
    }

    state.report_repo.create(payload)
        .await?;

    Ok((StatusCode::CREATED, response_nullable()))
}
