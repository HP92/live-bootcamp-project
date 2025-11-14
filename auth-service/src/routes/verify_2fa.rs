use axum::{extract::State, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use color_eyre::eyre::eyre;
use reqwest::StatusCode;
use secrecy::Secret;
use serde::Deserialize;

use crate::{
    app_state::AppState,
    domain::{AuthAPIError, Email, LoginAttemptId, TwoFACode},
    utils::generate_auth_cookie,
};

#[derive(Debug, Deserialize)]
pub struct Verify2FARequest {
    pub email: Secret<String>,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: Secret<String>,
    #[serde(rename = "2FACode")]
    pub two_fa_code: Secret<String>,
}

#[tracing::instrument(name = "Verify 2FA", skip_all)]
pub async fn verify_2fa(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<Verify2FARequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(request.email);
    let login_attempt_id = LoginAttemptId::parse(request.login_attempt_id);
    let two_fa_code = TwoFACode::parse(request.two_fa_code);

    if email.is_err() || login_attempt_id.is_err() || two_fa_code.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }

    let validation_result = {
        let mut two_fa_code_store = state.two_fa_code_store.write().await;
        let two_fa_stored_code_result = two_fa_code_store.get_code(email.as_ref().unwrap()).await;

        // if no email found with some
        if two_fa_stored_code_result.is_err() {
            return (jar, Err(AuthAPIError::IncorrectCredentials));
        }

        // Compare stored code with provided code and login attempt id
        let (stored_login_attempt_id, stored_two_fa_code) = two_fa_stored_code_result.unwrap();
        if &stored_login_attempt_id != login_attempt_id.as_ref().unwrap()
            || &stored_two_fa_code != two_fa_code.as_ref().unwrap()
        {
            return (jar, Err(AuthAPIError::IncorrectCredentials));
        }

        two_fa_code_store
            .remove_code(email.as_ref().unwrap().clone())
            .await
    };

    if validation_result.is_err() {
        return (
            jar,
            Err(AuthAPIError::UnexpectedError(eyre!(
                "Failed to remove 2FA code"
            ))),
        );
    }

    let auth_cookie = generate_auth_cookie(email.as_ref().unwrap()).unwrap();
    if auth_cookie.value().is_empty() {
        return (
            jar,
            Err(AuthAPIError::UnexpectedError(eyre!(
                "Failed to generate auth cookie"
            ))),
        );
    }

    (jar.add(auth_cookie), Ok(StatusCode::OK.into_response()))
}
