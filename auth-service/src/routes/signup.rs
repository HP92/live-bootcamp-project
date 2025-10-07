use reqwest::StatusCode;
use axum::response::IntoResponse;

pub async fn signup() -> impl IntoResponse {
    StatusCode::OK.into_response()
}
