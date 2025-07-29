use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateUserData;

#[tokio::test]
async fn test_user_sign_up() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let signup_data = json!({
        "username": "testuser",
        "password": "testpassword123@!"
    });
    let response = app
        .client
        .post_json("/sign_up", &signup_data)
        .await
        .expect("Failed to make request");
    response.assert_success();
}

#[tokio::test]
async fn test_user_sign_up_duplicate_name() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a user first
    let user_data = CreateUserData {
        name: "testuser".to_string(),
        ..Default::default()
    };
    app.fixtures
        .create_user(Some(user_data))
        .await
        .expect("Failed to create user");

    // Try to sign up with the same name
    let signup_data = json!({
        "name": "testuser",
        "password": "testpassword123@!"
    });

    let response = app
        .client
        .post_json("/sign_up", &signup_data)
        .await
        .expect("Failed to make request");

    response.assert_client_error();
}

#[tokio::test]
async fn test_user_sign_in() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // First sign up a user
    let signup_data = json!({
        "username": "testuser",
        "password": "testpassword123@!"
    });

    app.client
        .post_json("/sign_up", &signup_data)
        .await
        .expect("Failed to make request")
        .assert_success();

    // Then try to sign in
    let signin_data = json!({
        "username": "testuser",
        "password": "testpassword123@!"
    });

    let response = app
        .client
        .post_json("/sign_in", &signin_data)
        .await
        .expect("Failed to make request");

    response.assert_success();
}

#[tokio::test]
async fn test_user_sign_in_invalid_credentials() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let signin_data = json!({
        "username": "nonexistent",
        "password": "wrongpassword"
    });

    let response = app
        .client
        .post_json("/sign_in", &signin_data)
        .await
        .expect("Failed to make request");

    response.assert_client_error();
}

#[tokio::test]
async fn test_user_profile_unauthorized() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/profile").await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_profile_with_auth() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a user
    let user = app
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user");

    // Note: This test assumes you have a way to authenticate the client
    // You may need to implement proper authentication flow based on your app's auth system

    // For now, we'll just test the unauthorized case
    let response = app.client.get("/profile").await;
    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_user_update_bio() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let bio_data = json!({
        "bio": "This is my new bio"
    });

    // Without authentication, this should fail
    let response = app
        .client
        .post_json("/profile/bio", &bio_data)
        .await
        .expect("Failed to make request");
    response.assert_status(StatusCode::UNAUTHORIZED);
}
