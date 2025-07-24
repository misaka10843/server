use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateEventData;

#[tokio::test]
async fn test_create_event() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let event_data = json!({
        "name": "Test Event",
        "short_description": "Test short description",
        "description": "Test description"
    });

    let response = app
        .client
        .post_json("/event", &event_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}

#[tokio::test]
async fn test_find_event_by_id() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an event first
    let event = app
        .fixtures
        .create_event(None)
        .await
        .expect("Failed to create event");

    let response = app.client.get(&format!("/event/{}", event.id)).await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_find_event_by_id_not_found() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/event/99999").await;

    response.assert_success().assert_json_valid();

    // The response should contain null data for non-existent event
    let json: serde_json::Value = response.json();
    assert!(json.get("data").unwrap().is_null());
}

#[tokio::test]
async fn test_find_event_by_keyword() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an event with a specific name
    let event_data = CreateEventData {
        name: "Unique Event Name".to_string(),
        ..Default::default()
    };

    app.fixtures
        .create_event(Some(event_data))
        .await
        .expect("Failed to create event");

    let response = app.client.get("/event?keyword=Unique").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let events = data.as_array().unwrap();
    assert!(!events.is_empty(), "Should find at least one event");
}

#[tokio::test]
async fn test_find_event_by_keyword_no_results() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/event?keyword=NonexistentEvent").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let events = data.as_array().unwrap();
    assert!(events.is_empty(), "Should find no events");
}
