use reqwest::StatusCode;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{domain::AuthAPIError, utils::validate_token };

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct VerifyTokenRequest {
    pub token: String,
}

pub async fn verify_token( 
    Json(request): Json<VerifyTokenRequest>
) -> Result<impl IntoResponse, AuthAPIError> {
    
    if validate_token(&request.token).await.is_err(){
        return Err(AuthAPIError::InvalidToken);
    }

    Ok(StatusCode::OK.into_response())
}
