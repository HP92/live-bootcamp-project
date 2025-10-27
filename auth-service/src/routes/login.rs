use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{
    domain::{AuthAPIError, Email, LoginAttemptId, Password, TwoFACode},
    utils::auth::generate_auth_cookie,
    AppState,
};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginResponse2FA {
    pub message: String,
    #[serde(rename = "loginAttemptId")]
    pub login_attempt_id: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum LoginResponse {
    RegularAuth,
    TwoFactorAuth(LoginResponse2FA),
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(request): Json<LoginRequest>,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let email = Email::parse(&request.email);
    let password = Password::parse(&request.password);
    let user_store = state.user_store.write().await;

    if email.is_err() || password.is_err() {
        return (jar, Err(AuthAPIError::InvalidCredentials));
    }

    if (user_store
        .validate_user(email.as_ref().unwrap(), password.as_ref().unwrap())
        .await)
        .is_err()
    {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = user_store.get_user(email.as_ref().unwrap()).await.unwrap();
    match user.requires_2fa {
        true => handle_2fa(jar, &state, &user.email).await,
        false => handle_no_2fa(jar, &user.email).await,
    }
}

async fn handle_2fa(
    jar: CookieJar,
    state: &AppState,
    email: &Email,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let login_attempt_id = LoginAttemptId::default();
    let two_fa_code = TwoFACode::default();
    let mut two_fa_store = state.two_fa_code_store.write().await;

    if two_fa_store
        .add_code(email.clone(), login_attempt_id.clone(), two_fa_code.clone())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    let email_client = state.email_client_type.write().await;
    if email_client
        .send_email(
            email,
            "Your Authentication temporary code",
            two_fa_code.as_ref(),
        )
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::UnexpectedError));
    };

    let response = LoginResponse2FA {
        message: "2FA required".to_string(),
        login_attempt_id: login_attempt_id.as_ref().to_string(),
    };

    (
        jar,
        Ok((
            StatusCode::PARTIAL_CONTENT,
            Json(LoginResponse::TwoFactorAuth(response)),
        )),
    )
}

async fn handle_no_2fa(
    jar: CookieJar,
    email: &Email,
) -> (
    CookieJar,
    Result<(StatusCode, Json<LoginResponse>), AuthAPIError>,
) {
    let auth_cookie = generate_auth_cookie(email).unwrap();
    if auth_cookie.value().is_empty() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }

    (
        jar.add(auth_cookie),
        Ok((StatusCode::OK, Json(LoginResponse::RegularAuth))),
    )
}
