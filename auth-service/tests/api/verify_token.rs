use auth_service::{utils::JWT_COOKIE_NAME, ErrorResponse};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn verify_token_returns_200() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    // Create a user
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
    assert_eq!(login_response.status(), 200);

    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");
    assert!(!auth_cookie.value().is_empty());

    let test_case = serde_json::json!(
        {
            "token": auth_cookie.value(),
        }
    );
    let response = app.post_verify_token(&test_case).await;

    assert_eq!(response.status(), 200);
    app.clean_up().await;
}

#[tokio::test]
async fn verify_token_returns_401_for_invalid_token() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let signup_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
        "requires2FA": false
    });

    let response = app.post_signup(&signup_body).await;

    assert_eq!(response.status().as_u16(), 201);

    let login_body = serde_json::json!({
        "email": random_email,
        "password": "password123",
    });

    let response = app.post_login(&login_body).await;

    assert_eq!(response.status().as_u16(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());

    let token = auth_cookie.value();

    let response = app.post_logout().await;

    assert_eq!(response.status().as_u16(), 200);

    // ---------------------------------------------------------

    let verify_token_body = serde_json::json!({
        "token": token,
    });

    let response = app.post_verify_token(&verify_token_body).await;

    assert_eq!(response.status().as_u16(), 401);

    assert_eq!(
        response
            .json::<ErrorResponse>()
            .await
            .expect("Could not deserialize response body to ErrorResponse")
            .error,
        "Invalid token".to_owned()
    );
    app.clean_up().await;
}

#[tokio::test]
async fn verify_token_returns_422_for_malformed_input() {
    let app = TestApp::new().await;
    let test_cases = [
        serde_json::json!(
            {
                "token": "",
            }
        ),
        serde_json::json!({}),
        serde_json::json!(
            {
                "email": get_random_email(),
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}
