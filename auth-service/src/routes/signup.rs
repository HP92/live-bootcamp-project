use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{AppState, domain::User};

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
) -> impl IntoResponse {
    let user = User::new(request.email, request.password, request.requires_2fa);
    let mut user_store = state.user_store.write().await;

    let response;
    let status_code;
    if let Ok(_result) = user_store.add_user(user) {
        response = Json(SignupResponse {
            message: "User created successfully".to_string()
        });
        status_code = StatusCode::CREATED;
    } else {
        response = Json(SignupResponse {
            message: "User already exist".to_string()
        });
        status_code = StatusCode::CONFLICT;
    }

    println!("Response #{:?}  #{:?}", response, status_code);

    (status_code, response)
}
