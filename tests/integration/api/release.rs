use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateReleaseData;

#[tokio::test]
async fn test_create_release() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let release_data = json!({
        "title": "Test Release",
        "release_type": "Album"
    });

    let response = app
        .client
        .post_json("/release", &release_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}

#[tokio::test]
async fn test_find_release_by_id() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a release first
    let release = app
        .fixtures
        .create_release(None)
        .await
        .expect("Failed to create release");

    let response = app.client.get(&format!("/release/{}", release.id)).await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_find_release_by_id_not_found() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/release/99999").await;

    response.assert_success().assert_json_valid();

    // The response should contain null data for non-existent release
    let json: serde_json::Value = response.json();
    assert!(json.get("data").unwrap().is_null());
}

#[tokio::test]
async fn test_find_release_by_keyword() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a release with a specific title
    let release_data = CreateReleaseData {
        title: "Unique Release Title".to_string(),
        ..Default::default()
    };

    app.fixtures
        .create_release(Some(release_data))
        .await
        .expect("Failed to create release");

    let response = app.client.get("/release?keyword=Unique").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let releases = data.as_array().unwrap();
    assert!(!releases.is_empty(), "Should find at least one release");
}

#[tokio::test]
async fn test_find_release_by_keyword_no_results() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/release?keyword=NonexistentRelease").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let releases = data.as_array().unwrap();
    assert!(releases.is_empty(), "Should find no releases");
}

#[tokio::test]
async fn test_update_release() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a release first
    let release = app
        .fixtures
        .create_release(None)
        .await
        .expect("Failed to create release");

    let update_data = json!({
        "title": "Updated Release Title",
        "release_type": "Single"
    });

    let response = app
        .client
        .put_json(&format!("/release/{}", release.id), &update_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}
