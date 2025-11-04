use crate::helpers::{get_random_email, TestApp};

use auth_service::{domain::Email, routes::LoginResponse2FA, utils::constants::JWT_COOKIE_NAME};

#[tokio::test]
async fn login_returns_200_if_valid_credentials_and_2fa_disabled() {
    let app = TestApp::new().await;

    let first_input = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
            "requires2FA": false
        }
    );
    let _ = app.post_signup(&first_input).await;

    let test_case = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
        }
    );

    let response = app.post_login(&test_case).await;

    assert_eq!(response.status(), 200);

    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}

#[tokio::test]
async fn login_returns_206_if_valid_credentials_and_2fa_enabled() {
    let test_case = serde_json::json!(
        {
            "email": "test@example.com",
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let app = TestApp::new().await;
    let response = app.post_signup(&test_case).await;

    assert_eq!(response.status().as_u16(), 201);

    let test_case = serde_json::json!(
        {
            "email": "test@example.com",
            "password": "Asdf1234@",
        }
    );

    let response = app.post_login(&test_case).await;
    assert_eq!(response.status(), 206);

    let response_body = response
        .json::<LoginResponse2FA>()
        .await
        .expect("Could not deserialize response body to LoginResponse2FA");

    assert_eq!(response_body.message, "2FA required".to_owned());

    let login_attempt_id_len = response_body.login_attempt_id.clone();
    let two_fa_store = app.two_fa_code_store.read().await;
    let example_email = Email::parse("test@example.com");

    assert_eq!(
        login_attempt_id_len,
        two_fa_store
            .get_code(&example_email.unwrap())
            .await
            .unwrap()
            .0
            .as_ref()
            .to_string()
    );
    app.clean_up().await;
}

#[tokio::test]
async fn login_returns_400_if_invalid_input() {
    let test_cases = [
        serde_json::json!(
            {
                "email": "test.com",
                "password": "password123",
            }
        ),
        serde_json::json!(
            {
                "email": "example@test.com",
                "password": "1234567",
            }
        ),
    ];
    let app = TestApp::new().await;

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn login_returns_401_if_invalid_credentials() {
    let app = TestApp::new().await;
    // Create a user first
    let first_input = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
            "requires2FA": true
        }
    );
    let _ = app.post_signup(&first_input).await;

    // Now try to login with wrong password
    let second_input = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf12345",
        }
    );

    let response = app.post_login(&second_input).await;
    assert_eq!(
        response.status().as_u16(),
        401,
        "Failed for input: {:?}",
        second_input
    );
    app.clean_up().await;
}

#[tokio::test]
async fn login_returns_422_if_malformed_credentials() {
    let app = TestApp::new().await;

    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!(
            {
                "password": "password123",
            }
        ),
        serde_json::json!(
            {
                "email": random_email
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_login(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}
