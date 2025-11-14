use axum::{extract::State, response::IntoResponse, Json};
use reqwest::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::{app_state::AppState, domain::AuthAPIError, utils::validate_token};

#[derive(Debug, Deserialize)]
pub struct VerifyTokenRequest {
    pub token: Secret<String>,
}

#[tracing::instrument(name = "Verify Token", skip_all)]
pub async fn verify_token(
    State(state): State<AppState>,
    Json(request): Json<VerifyTokenRequest>,
) -> Result<impl IntoResponse, AuthAPIError> {
    if validate_token(state.banned_token_store, request.token)
        .await
        .is_err()
    {
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}
