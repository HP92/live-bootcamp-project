use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use axum_extra::extract::CookieJar;
use serde::{Deserialize, Serialize};

use crate::{domain::{AuthAPIError, Email, Password, UserStore}, AppState, utils::auth::generate_auth_cookie};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
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

    if let Err(_) = user_store.validate_user(&email.as_ref().unwrap(), &password.as_ref().unwrap()).await {
        return (jar, Err(AuthAPIError::IncorrectCredentials));
    }

    let user = user_store.get_user(&email.as_ref().unwrap()).await.unwrap();
    let auth_cookie = generate_auth_cookie(&user.email).unwrap();

    if auth_cookie.value().is_empty() {
        return (jar, Err(AuthAPIError::UnexpectedError));
    }
    
    (jar.add(auth_cookie), Ok(StatusCode::OK.into_response()))
}