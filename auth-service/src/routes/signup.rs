use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, Email, User, UserStore, Password}, AppState};

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
    State(state): State<Arc<AppState>>,
    Json(request): Json<SignupRequest>
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email);
    let password = Password::parse(&request.password);
    let mut user_store = state.user_store.write().await;
    let result;
    if email.is_ok() && password.is_ok() {
        let user = User::new(email.unwrap(), password.unwrap(), request.requires_2fa);
        if let Ok(_result) = user_store.add_user(user).await {
            let response = Json(SignupResponse {
                message: "User created successfully".to_string()
            });
            let status_code = StatusCode::CREATED;
            result = Ok((status_code, response))
        } else {
            result = Err(AuthAPIError::UserAlreadyExists)
        }
    } else {
        result = Err(AuthAPIError::InvalidCredentials)
    }

    result
}
