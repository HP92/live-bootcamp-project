use auth_service::{
    domain::{Email, LoginAttemptId, TwoFACode},
    utils::JWT_COOKIE_NAME,
};
use secrecy::{ExposeSecret, Secret};
use uuid::Uuid;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{get_random_email, TestApp};

#[tokio::test]
async fn verify_2fa_returns_200_if_correct_code() {
    let app: TestApp = TestApp::new().await;

    // Set up mock BEFORE calling setup_user_for_verify_2fa (which sends email during login)
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let random_email = get_random_email();
    let (login_attempt_id, two_fa_code) =
        setup_user_for_verify_2fa(&app, random_email.clone()).await;

    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().expose_secret().to_string(),
            "2FACode": two_fa_code.as_ref().expose_secret().to_string()
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

    // Set up mock BEFORE calling setup_user_for_verify_2fa (which sends email during login)
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let random_email = get_random_email();
    let (login_attempt_id, two_fa_code) =
        setup_user_for_verify_2fa(&app, random_email.clone()).await;

    let test_cases = [
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": login_attempt_id.as_ref().expose_secret().to_string(),
                "2FACode": "123456" // incorrect code,
            }
        ),
        serde_json::json!(
            {
                "email": random_email.clone(),
                "loginAttemptId": Uuid::new_v4(), // incorrect login attempt id
                "2FACode": two_fa_code.as_ref().expose_secret().to_string() ,
            }
        ),
        serde_json::json!(
            {
                "email": get_random_email(), // incorrect email
                "loginAttemptId": login_attempt_id.as_ref().expose_secret().to_string(),
                "2FACode": two_fa_code.as_ref().expose_secret().to_string(),
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

    // Set up mock BEFORE any login calls (first login in setup + second login below = 2 emails)
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(2)
        .mount(&app.email_server)
        .await;

    let random_email = get_random_email();
    let (first_login_attempt_id, first_two_fa_code) =
        setup_user_for_verify_2fa(&app, random_email.clone()).await;

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
            "loginAttemptId": first_login_attempt_id.as_ref().expose_secret().to_string(), // using old login attempt id
            "2FACode": first_two_fa_code.as_ref().expose_secret().to_string()
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
    let (login_attempt_id, two_fa_code) =
        setup_user_for_verify_2fa(&app, random_email.clone()).await;
    let test_case = serde_json::json!(
        {
            "email": random_email,
            "loginAttemptId": login_attempt_id.as_ref().expose_secret().to_string(),
            "2FACode": two_fa_code.as_ref().expose_secret().to_string()
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
            "loginAttemptId": login_attempt_id.as_ref().expose_secret().to_string(),
            "2FACode": two_fa_code.as_ref().expose_secret().to_string()
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
                "2FACode": "789012",
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

async fn setup_user_for_verify_2fa(app: &TestApp, email: String) -> (LoginAttemptId, TwoFACode) {
    let create_account = serde_json::json!(
        {
            "email": email.clone(),
            "password": "Asdf1234@",
            "requires2FA": true
        }
    );
    let _ = app.post_signup(&create_account).await;

    let login_user = serde_json::json!(
        {
            "email": email.clone(),
            "password": "Asdf1234@",
        }
    );
    let _ = app.post_login(&login_user).await;

    let example_email = Email::parse(Secret::new(email.clone()));
    get_two_fa_code_and_login_attemp(&app, example_email.as_ref().unwrap()).await
}

// To avoid locking the resource I am recreating this function to have a smaller scope
// for the two_fa_code_store
async fn get_two_fa_code_and_login_attemp(
    app: &TestApp,
    email: &Email,
) -> (LoginAttemptId, TwoFACode) {
    app.two_fa_code_store
        .write()
        .await
        .get_code(email)
        .await
        .unwrap()
}
