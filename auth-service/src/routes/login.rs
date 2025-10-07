use reqwest::StatusCode;
use axum::response::IntoResponse;

pub async fn login() -> impl IntoResponse {
    StatusCode::OK.into_response()
}