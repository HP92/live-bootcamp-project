use auth_service::routes::SignupResponse;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn signup_returns_201_if_valid_input() {
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

    let expected_response = SignupResponse {
        message: "User created successfully".to_owned(),
    };

    assert_eq!(
        response
            .json::<SignupResponse>()
            .await
            .expect("Could not deserialize response body to UserBody"),
        expected_response
    );
}

#[tokio::test]
async fn singup_returns_400_if_invalid_input() {
    let test_cases = [
        serde_json::json!(
            {
                "email": "test.com",
                "password": "password123",
                "requires2FA": true
            }
        ),
        serde_json::json!(
            {
                "email": "example@test.com",
                "password": "1234567",
                "requires2FA": true
            }
        ),
    ];
    let app = TestApp::new().await;

    for test_case in test_cases.iter() {
        let response = app.post_signup(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            400,
            "Failed for input: {:?}",
            test_case
        );
    }
}

#[tokio::test]
async fn signup_returns_409_user_already_exist() {
    let app = TestApp::new().await;
    let first_input = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&first_input).await;
    assert_eq!(
        response.status().as_u16(),
        201,
        "Failed for input: {:?}",
        first_input
    );

    let second_input = serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
            "requires2FA": true
        }
    );

    let response = app.post_signup(&second_input).await;
    assert_eq!(
        response.status().as_u16(),
        409,
        "Failed for input: {:?}",
        second_input
    );
}

#[tokio::test]
async fn signup_returns_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!(
            {
                "password": "password123",
                "requires2FA": true
            }
        ),
        serde_json::json!(
            {
                "password": "password123",
                "email": random_email
            }
        ),
        serde_json::json!(
            {
                "email": random_email,
                "requires2FA": true
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
}
