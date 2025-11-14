use axum::{extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use color_eyre::eyre::Result;
use secrecy::Secret;

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

    let token = Secret::new(cookie.unwrap().value().to_string());

    // Validate the token (checks if it's banned and if it's properly formatted/valid)
    if validate_token(state.banned_token_store.clone(), token.clone())
        .await
        .is_err()
    {
        return (jar, Err(AuthAPIError::InvalidToken));
    }

    // Add the token to banned list
    let result = {
        let mut banned_token_store = state.banned_token_store.write().await;
        banned_token_store.add_token(token).await
    };

    if let Err(e) = result {
        return (jar, Err(AuthAPIError::UnexpectedError(e)));
    }

    let jar = jar.remove(Cookie::from(JWT_COOKIE_NAME));

    (jar, Ok(StatusCode::OK))
}
