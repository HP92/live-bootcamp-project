use crate::helpers::TestApp;

#[tokio::test]
async fn root_returns_auth_ui() {
    let app = TestApp::new().await;
    let response = app.get_root().await;

    assert_eq!(response.status(), 200);
    app.clean_up().await;
}
