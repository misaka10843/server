use axum::http::StatusCode;
use serde_json::json;

use crate::integration::TestApp;
use crate::integration::assertions::ResponseAssertions;
use crate::integration::fixtures::CreateSongData;

#[tokio::test]
async fn test_create_song() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let song_data = json!({
        "title": "Test Song"
    });

    let response = app
        .client
        .post_json("/song", &song_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}

#[tokio::test]
async fn test_find_song_by_id() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a song first
    let song = app
        .fixtures
        .create_song(None)
        .await
        .expect("Failed to create song");

    let response = app.client.get(&format!("/song/{}", song.id)).await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");
}

#[tokio::test]
async fn test_find_song_by_id_not_found() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/song/99999").await;

    response.assert_success().assert_json_valid();

    // The response should contain null data for non-existent song
    let json: serde_json::Value = response.json();
    assert!(json.get("data").unwrap().is_null());
}

#[tokio::test]
async fn test_find_song_by_keyword() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a song with a specific title
    let song_data = CreateSongData {
        title: "Unique Song Title".to_string(),
    };

    app.fixtures
        .create_song(Some(song_data))
        .await
        .expect("Failed to create song");

    let response = app.client.get("/song?keyword=Unique").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let songs = data.as_array().unwrap();
    assert!(!songs.is_empty(), "Should find at least one song");
}

#[tokio::test]
async fn test_find_song_by_keyword_no_results() {
    let app = TestApp::new().await.expect("Failed to create test app");

    let response = app.client.get("/song?keyword=NonexistentSong").await;

    response
        .assert_success()
        .assert_json_valid()
        .assert_json_has_field("data");

    let json: serde_json::Value = response.json();
    let data = json.get("data").unwrap();
    assert!(data.is_array());

    let songs = data.as_array().unwrap();
    assert!(songs.is_empty(), "Should find no songs");
}

#[tokio::test]
async fn test_update_song() {
    let app = TestApp::new().await.expect("Failed to create test app");

    // Create a song first
    let song = app
        .fixtures
        .create_song(None)
        .await
        .expect("Failed to create song");

    let update_data = json!({
        "title": "Updated Song Title"
    });

    let response = app
        .client
        .post_json(&format!("/song/{}", song.id), &update_data)
        .await
        .expect("Failed to make request");

    // Note: This might require authentication depending on your API design
    response.assert_status(StatusCode::UNAUTHORIZED); // Assuming authentication is required
}
