use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, Email, User, Password}, AppState};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignupRequest {
    pub email: String,
    pub password: String,
    #[serde(rename = "requires2FA")]
    pub requires_2fa: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SignupResponse {
    pub message: String,
}

pub async fn signup(
    State(state): State<AppState>,
    Json(request): Json<SignupRequest>
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email);
    let password = Password::parse(&request.password);
    let mut user_store = state.user_store.write().await;

    if email.is_err() || password.is_err() {
        return Err(AuthAPIError::InvalidCredentials);
    }

    let user = User::new(email.unwrap(), password.unwrap(), request.requires_2fa);
    if let Err(_result) = user_store.add_user(user).await {
        return Err(AuthAPIError::UserAlreadyExists);
    }

    let response = Json(SignupResponse {
        message: "User created successfully".to_string()
    });

    Ok((StatusCode::CREATED, response))
}
