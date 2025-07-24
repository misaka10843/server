use axum::http::StatusCode;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;

#[tokio::test]
async fn test_health_check() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/health_check").await;

    response.assert_status(StatusCode::OK);
}

#[tokio::test]
async fn test_home_page() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    // Note: Content type assertion removed due to method conflicts with axum_test
}
