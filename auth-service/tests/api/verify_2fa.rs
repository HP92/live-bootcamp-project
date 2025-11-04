use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode},
    routes::LoginResponse2FA,
    utils::JWT_COOKIE_NAME,
};
use uuid::Uuid;

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn verify_2fa_returns_200_if_correct_code() {
    let app: TestApp = TestApp::new().await;
    let random_email = get_random_email();
    let create_account = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&create_account).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_user = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
        }
    );
    let response = app.post_login(&login_user).await;
    assert_eq!(response.status(), 206);

    let example_email = Email::parse(&random_email.clone());
    let (login_attempt_id, two_fa_code) =
        get_two_fa_code_and_login_attemp(&app, example_email.as_ref().unwrap()).await;
    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": login_attempt_id,
            "2FACode": two_fa_code.as_ref().to_string()
        }
    );
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 200);
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    app.clean_up().await;
}

#[tokio::test]
async fn verify_2fa_returns_400_if_invalid_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": "123456",
                "2FACode": "invalid_uuid"
            }
        ),
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": "12345",
                "2FACode": Uuid::new_v4().to_string()
            }
        ),
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": "abcdef",
                "2FACode": Uuid::new_v4().to_string()
            }
        ),
        serde_json::json!(
            {
                "email": "test.com",
                "loginAttemptId": "123456",
                "2FACode": Uuid::new_v4().to_string()
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
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
async fn verify_2fa_returns_401_if_incorect_credentials() {
    let app: TestApp = TestApp::new().await;

    let random_email = get_random_email();
    let create_account = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&create_account).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_user = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
        }
    );
    let response = app.post_login(&login_user).await;
    assert_eq!(response.status(), 206);
    let _response_body = response
        .json::<LoginResponse2FA>()
        .await
        .expect("Could not deserialize response body to LoginResponse2FA");
    let example_email = Email::parse(&random_email.clone());

    let (login_attempt_id, two_fa_code) =
        get_two_fa_code_and_login_attemp(&app, example_email.as_ref().unwrap()).await;

    let test_cases = [
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": login_attempt_id.clone(),
                "2FACode": "123456" // incorrect code,
            }
        ),
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": Uuid::new_v4(), // incorrect login attempt id
                "2FACode": two_fa_code.as_ref().to_string() ,
            }
        ),
        serde_json::json!(
            {
                "email": get_random_email(), // incorrect email
                "loginAttemptId":login_attempt_id,
                "2FACode": two_fa_code.as_ref().to_string(),
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            401,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}

#[tokio::test]
async fn verify_2fa_returns_401_if_old_code() {
    let app: TestApp = TestApp::new().await;

    let random_email = get_random_email();
    let create_account = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&create_account).await;
    assert_eq!(response.status().as_u16(), 201);

    // First login to generate 2FA code
    let first_login = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
        }
    );
    let response = app.post_login(&first_login).await;
    assert_eq!(response.status(), 206);

    let example_email = Email::parse(&random_email.clone());
    let (first_login_attempt_id, first_two_fa_code) =
        get_two_fa_code_and_login_attemp(&app, example_email.as_ref().unwrap()).await;

    // Second login to overwrite 2FA code
    let second_login = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
        }
    );
    let response = app.post_login(&second_login).await;
    assert_eq!(response.status(), 206);

    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": first_login_attempt_id, // using old login attempt id
            "2FACode": first_two_fa_code.as_ref().to_string()
        }
    );
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn should_return_401_if_same_code_twice() {
    let app: TestApp = TestApp::new().await;
    let random_email = get_random_email();
    let create_account = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let response = app.post_signup(&create_account).await;
    assert_eq!(response.status().as_u16(), 201);

    let login_user = serde_json::json!(
        {
            "email": random_email.clone(),
            "password": "Asdf1234@",
        }
    );
    let response = app.post_login(&login_user).await;
    assert_eq!(response.status(), 206);

    let example_email = Email::parse(&random_email.clone());
    let (login_attempt_id, two_fa_code) =
        get_two_fa_code_and_login_attemp(&app, example_email.as_ref().unwrap()).await;
    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().to_string(),
            "2FACode": two_fa_code.as_ref().to_string()
        }
    );
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 200);
    let auth_cookie = response
        .cookies()
        .find(|cookie| cookie.name() == JWT_COOKIE_NAME)
        .expect("No auth cookie found");

    assert!(!auth_cookie.value().is_empty());
    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().to_string(),
            "2FACode": two_fa_code.as_ref().to_string()
        }
    );
    let response = app.post_verify_2fa(&test_case).await;
    assert_eq!(response.status(), 401);
    app.clean_up().await;
}

#[tokio::test]
async fn verify_2fa_returns_422_if_malformed_input() {
    let app = TestApp::new().await;
    let random_email = get_random_email();
    let test_cases = [
        serde_json::json!(
            {
                "email": random_email,
                "loginAttemptId": "random_login_attempt_id"
            }
        ),
        serde_json::json!(
            {
                "2FACode": "123456",
                "email": random_email
            }
        ),
        serde_json::json!(
            {
                "2FACode": "123456",
                "loginAttemptId": "random_login_attempt_id"
            }
        ),
    ];

    for test_case in test_cases.iter() {
        let response = app.post_verify_2fa(test_case).await;
        assert_eq!(
            response.status().as_u16(),
            422,
            "Failed for input: {:?}",
            test_case
        );
    }
    app.clean_up().await;
}

// To avoid locking the resource I am recreating this function to have a smaller scope
// for the two_fa_code_store
async fn get_two_fa_code_and_login_attemp(
    app: &TestApp,
    email: &Email,
) -> (LoginAttemptId, TwoFACode) {
    let two_fa_code_store = app.two_fa_code_store.read().await;
    two_fa_code_store.get_code(email).await.unwrap()
}
