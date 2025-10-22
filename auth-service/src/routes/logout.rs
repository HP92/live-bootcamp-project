use axum::{http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};

use crate::{
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

pub async fn logout(jar: CookieJar) -> (CookieJar ,Result<impl IntoResponse, AuthAPIError>) {
    let cookie = jar.get(JWT_COOKIE_NAME);
    if cookie.is_none() {
        return (jar, Err(AuthAPIError::MissingToken));
    }

    let token = cookie.unwrap().value();
    if validate_token(token).await.is_err() {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
