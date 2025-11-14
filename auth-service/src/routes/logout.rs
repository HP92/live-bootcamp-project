use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use color_eyre::eyre::Result;

use crate::{
    app_state::AppState,
    domain::AuthAPIError,
    utils::{auth::validate_token, constants::JWT_COOKIE_NAME},
};

#[tracing::instrument(name = "Logout", skip_all)]
pub async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
) -> (CookieJar, Result<impl IntoResponse, AuthAPIError>) {
    let cookie = jar.get(JWT_COOKIE_NAME);
    if cookie.is_none() {
        return (jar, Err(AuthAPIError::MissingToken));
    }

    let token = cookie.unwrap().value();
    if validate_token(state.banned_token_store.clone(), token)
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    if let Err(e) = state.banned_token_store.write().await.add_token(token.to_string()).await {
        return (jar, Err(AuthAPIError::UnexpectedError(e)));
    }

    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
