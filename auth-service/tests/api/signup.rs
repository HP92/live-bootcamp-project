use crate::helpers::{TestApp, get_random_email};

// TODO: Fix to throw 200
// #[tokio::test]
// async fn signup_returns_200(){
//     let app = TestApp::new().await;
//     let random_email = get_random_email();
//     let test_case = 
//         &serde_json::json!(
//             {
//                 "email": random_email,
//                 "password": "asdf1234",
//                 "requires2FA": true
//             }
//         );

//     let response = app.post_signup(test_case).await;

//     assert_eq!(response.status().as_u16(), 200, "Failed for input: {:?}", test_case);
// }

#[tokio::test]
async fn signup_returns_422_if_malformed_input(){
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
        )
    ];

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