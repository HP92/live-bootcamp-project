use auth_service::utils::JWT_COOKIE_NAME;

use crate::helpers::{TestApp, get_random_email};

#[tokio::test]
async fn verify_token_returns_200(){
    let app = TestApp::new().await;
    // Create a user
    let first_input = 
    serde_json::json!(
        {
            "email": "example@test.com",
            "password": "asdf1234",
            "requires2FA": false
        }
    );
    let _ = app.post_signup(&first_input).await;

    // Login with the user
    let second_input = 
        serde_json::json!(
            {
                "email": "example@test.com",
                "password": "asdf1234",
            }
        );
    let login_response = app.post_login(&second_input).await;
    let auth_cookie = login_response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

     assert!(!auth_cookie.value().is_empty());
    
    let test_case = 
        serde_json::json!(
            {
                "token": auth_cookie.value(),
            }
        );
    let response = app.post_verify_token(&test_case).await;

    assert_eq!(response.status(), 200);
}
    
#[tokio::test]
async fn verify_token_returns_401_for_invalid_token(){
    let app = TestApp::new().await;
    let test_case = 
        serde_json::json!(
            {
                "token": "invalid_token",
            }
        );
    let response = app.post_verify_token(&test_case).await;

    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn verify_token_returns_422_for_malformed_input(){
    let app = TestApp::new().await;
    let test_cases = [
        serde_json::json!(
            {
                "token": "",
            }
        ),
        serde_json::json!(
            {
            }
        ),
        serde_json::json!(
            {
                "email": get_random_email(),
            }
        )
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