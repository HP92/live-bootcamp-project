use reqwest::Url;

use auth_service::utils::JWT_COOKIE_NAME;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn logout_returns_200_logout_succesful() {
    let app = TestApp::new().await;

    match setup_user_for_logout(&app).await {
        Ok(auth_cookie) => {
            let response = app.post_logout().await;
            assert_eq!(response.status(), 200);

            let mut banned_token_store = app.banned_token_store.write().await;
            let is_token_banned = banned_token_store
                .contains_token(auth_cookie.as_ref())
                .await
                .expect("Failed to check if token is banned");

            assert!(is_token_banned);
        }
        Err(e) => {
            panic!("Failed to set up user for logout: {}", e);
        }
    }
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_400_if_logout_called_twice_in_a_row() {
    let app = TestApp::new().await;

    match setup_user_for_logout(&app).await {
        Ok(_) => {
            let response = app.post_logout().await;
            assert_eq!(response.status(), 200);

            let response = app.post_logout().await;
            assert_eq!(response.status(), 400);
        }
        Err(e) => {
            panic!("Failed to set up user for logout: {}", e);
        }
    }

    app.clean_up().await;
}

#[tokio::test]
async fn logout_returns_400_if_jwt_cookie_missing() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &"HttpOnly; SameSite=Lax; Secure; Path=/",
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), 400);
    app.clean_up().await;
}

#[tokio::test]
async fn logout_returns_401_if_invalid_token() {
    let app = TestApp::new().await;

    app.cookie_jar.add_cookie_str(
        &format!(
            "{}=invalid; HttpOnly; SameSite=Lax; Secure; Path=/",
            JWT_COOKIE_NAME
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    let response = app.post_logout().await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

async fn setup_user_for_logout(app: &TestApp) -> Result<String, String> {
    // Create a user
    let random_email = get_random_email();

    let first_input = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "asdf1234",
            "requires2FA": false
        }
    );
    let _ = app.post_signup(&first_input).await;

    // Login with the user
    let second_input = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "asdf1234",
        }
    );
    let login_response = app.post_login(&second_input).await;

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    app.cookie_jar.add_cookie_str(
        &format!(
            "#{JWT_COOKIE_NAME}=#{}; HttpOnly; SameSite=Lax; Secure; Path=/",
            auth_cookie.value()
        ),
        &Url::parse("http://127.0.0.1").expect("Failed to parse URL"),
    );

    Ok(auth_cookie.value().to_owned())
}
