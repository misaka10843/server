use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateTagData;

#[tokio::test]
async fn test_create_tag() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let tag_data = json!({
        "name": "Test Tag",
        "type": "Genre",
        "short_description": "Test short description",
        "description": "Test description"
    });

    let response = app
        .client
        .post_json("/tag", &tag_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}

#[tokio::test]
async fn test_find_tag_by_id() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a tag first
    let tag = app
        .fixtures
        .create_tag(None)
        .await
        .expect("Failed to create tag");

    let response = app.client.get(&format!("/tag/{}", tag.id)).await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_find_tag_by_id_not_found() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/tag/99999").await;

    response.assert_success().assert_json_valid();

    // The response should contain null data for non-existent tag
    let json: serde_json::Value = response.json();
    assert!(json.get("data").unwrap().is_null());
}

#[tokio::test]
async fn test_find_tag_by_keyword() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a tag with a specific name
    let tag_data = CreateTagData {
        name: "Unique Tag Name".to_string(),
        ..Default::default()
    };

    app.fixtures
        .create_tag(Some(tag_data))
        .await
        .expect("Failed to create tag");

    let response = app.client.get("/tag?keyword=Unique").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let tags = data.as_array().unwrap();
    assert!(!tags.is_empty(), "Should find at least one tag");
}

#[tokio::test]
async fn test_find_tag_by_keyword_no_results() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/tag?keyword=NonexistentTag").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let tags = data.as_array().unwrap();
    assert!(tags.is_empty(), "Should find no tags");
}
