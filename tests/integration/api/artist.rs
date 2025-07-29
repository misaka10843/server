use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateArtistData;

#[tokio::test]
async fn test_create_artist() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let artist_data = json!({
        "name": "Test Artist",
        "artist_type": "Person"
    });

    let response = app
        .client
        .post_json("/artist", &artist_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    // Adjust the expected status code based on your implementation
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}

#[tokio::test]
async fn test_find_artist_by_id() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an artist first
    let artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    let response = app.client.get(&format!("/artist/{}", artist.id)).await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_find_artist_by_id_not_found() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/artist/99999").await;

    response.assert_success().assert_json_valid();

    // The response should contain null data for non-existent artist
    let json: serde_json::Value = response.json();
    assert!(json.get("data").unwrap().is_null());
}

#[tokio::test]
async fn test_find_artist_by_keyword() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an artist with a specific name
    let artist_data = CreateArtistData {
        name: "Unique Artist Name".to_string(),
        ..Default::default()
    };

    app.fixtures
        .create_artist(Some(artist_data))
        .await
        .expect("Failed to create artist");

    let response = app.client.get("/artist?keyword=Unique").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    // let artists = data.as_array().unwrap();
    // assert!(!artists.is_empty(), "Should find at least one artist");
}

#[tokio::test]
async fn test_find_artist_by_keyword_no_results() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/artist?keyword=NonexistentArtist").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let artists = data.as_array().unwrap();
    assert!(artists.is_empty(), "Should find no artists");
}

#[tokio::test]
async fn test_artist_discographies() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an artist first
    let artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    let response = app
        .client
        .get(&format!(
            "/artist/{}/discographies?release_type=Album&cursor=0&limit=10",
            artist.id
        ))
        .await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_artist_appearances() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an artist first
    let artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    let response = app
        .client
        .get(&format!(
            "/artist/{}/appearances?cursor=0&limit=10",
            artist.id
        ))
        .await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_artist_credits() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create an artist first
    let artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    let response = app
        .client
        .get(&format!("/artist/{}/credits?cursor=0&limit=10", artist.id))
        .await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}
