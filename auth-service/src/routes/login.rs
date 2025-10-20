use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, Email, Password, UserStore}, AppState};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginResponse {
    pub message: String,
}

pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>
) -> Result<impl IntoResponse, AuthAPIError> {
    let email = Email::parse(&request.email);
    let password = Password::parse(&request.password);
    let user_store = state.user_store.write().await;
    let result;

    if email.is_ok() && password.is_ok() {
        if let Ok(user) = user_store.get_user(&email.unwrap()).await {
            if user.password.eq(&password.unwrap()) {
                result = Ok(Json(LoginResponse {
                    message: "Login successful".to_string()
                }))
            } else {
                result = Err(AuthAPIError::IncorrectCredentials)
            }
        } else {
            result = Err(AuthAPIError::InvalidCredentials)
        }
    } else {
        result = Err(AuthAPIError::InvalidCredentials)
    }

    result
}