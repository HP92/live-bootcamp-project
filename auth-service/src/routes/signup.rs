use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, User, UserStore}, AppState};

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
    let user = User::new(request.email, request.password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    let result;
    if user.validate_email() && user.validate_password() {
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
