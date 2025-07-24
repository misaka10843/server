//! Comprehensive integration test
//!
//! This test provides comprehensive coverage of the integration test framework:
//! - Database setup and isolation
//! - HTTP API testing
//! - Entity creation and verification
//! - Fixture usage and custom data
//! - Proper resource cleanup

#[path = "integration/lib.rs"]
mod integration;

use entity::*;
use integration::fixtures::*;
use integration::{
    TestApp, assert_non_empty_string, assert_valid_id, patterns,
};
use sea_orm::{EntityTrait, PaginatorTrait};

#[tokio::test]
async fn test_basic_setup_and_endpoints() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    // Test basic HTTP endpoints
    patterns::test_basic_endpoints(&app)
        .await
        .expect("HTTP endpoints test failed");

    // Test that we can create test data
    let user = app
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create test user");

    assert_non_empty_string(&user.name, "User name");
    assert_valid_id(user.id, "User");

    // Test database reset functionality
    app.reset().await.expect("Failed to reset database");

    app.cleanup().await.expect("Failed to cleanup test app");
}

#[tokio::test]
async fn test_database_isolation() {
    let mut app1 = TestApp::new()
        .await
        .expect("Failed to create first test app");

    let _user1 = app1
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user in app1");

    let mut app2 = TestApp::new()
        .await
        .expect("Failed to create second test app");

    let _user2 = app2
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user in app2");

    // Verify that the apps are isolated (they should have different database URLs)
    assert_ne!(
        app1.database.url(),
        app2.database.url(),
        "Apps should use different databases"
    );

    app1.cleanup()
        .await
        .expect("Failed to cleanup first test app");
    app2.cleanup()
        .await
        .expect("Failed to cleanup second test app");
}

#[tokio::test]
async fn test_fixtures_and_entities() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    let artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    assert_non_empty_string(&artist.name, "Artist name");
    assert_valid_id(artist.id, "Artist");

    let song = app
        .fixtures
        .create_song(None)
        .await
        .expect("Failed to create song");

    assert_non_empty_string(&song.title, "Song title");
    assert_valid_id(song.id, "Song");

    let release = app
        .fixtures
        .create_release(None)
        .await
        .expect("Failed to create release");

    assert_non_empty_string(&release.title, "Release title");
    assert_valid_id(release.id, "Release");

    let tag = app
        .fixtures
        .create_tag(None)
        .await
        .expect("Failed to create tag");

    assert_non_empty_string(&tag.name, "Tag name");
    assert_valid_id(tag.id, "Tag");

    let event = app
        .fixtures
        .create_event(None)
        .await
        .expect("Failed to create event");

    assert_non_empty_string(&event.name, "Event name");
    assert_valid_id(event.id, "Event");

    app.cleanup().await.expect("Failed to cleanup test app");
}

#[tokio::test]
async fn test_entity_creation_and_verification() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    patterns::create_and_verify_entities(&app)
        .await
        .expect("Entity creation and verification failed");

    app.cleanup().await.expect("Failed to cleanup test app");
}

#[tokio::test]
async fn test_custom_fixture_data() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    let custom_user_data = CreateUserData {
        name: "custom_test_user".to_string(),
        password_hash: "$2b$12$custom_hash_for_testing".to_string(),
        bio: Some("Custom bio for testing".to_string()),
        ..Default::default()
    };

    let user = app
        .fixtures
        .create_user(Some(custom_user_data))
        .await
        .expect("Failed to create custom user");

    assert_eq!(user.name, "custom_test_user");
    assert_eq!(user.bio, Some("Custom bio for testing".to_string()));
    assert_valid_id(user.id, "Custom user");

    let custom_artist_data = CreateArtistData {
        name: "Custom Artist".to_string(),
        ..Default::default()
    };

    let artist = app
        .fixtures
        .create_artist(Some(custom_artist_data))
        .await
        .expect("Failed to create custom artist");

    assert_eq!(artist.name, "Custom Artist");
    assert_valid_id(artist.id, "Custom artist");

    app.cleanup().await.expect("Failed to cleanup test app");
}

#[tokio::test]
async fn test_database_operations_and_queries() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    // Create multiple entities
    let _user1 = app
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user1");
    let _user2 = app
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user2");
    let _artist = app
        .fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    // Test database queries
    let connection = app.database.connection();

    let user_count = user::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count users");
    assert_eq!(user_count, 2, "Should have 2 users");

    let artist_count = artist::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count artists");
    assert_eq!(artist_count, 1, "Should have 1 artist");

    // Test finding specific entities
    let found_user = user::Entity::find_by_id(1)
        .one(connection)
        .await
        .expect("Failed to find user")
        .expect("User should exist");
    assert!(!found_user.name.is_empty(), "User should have a name");

    app.cleanup().await.expect("Failed to cleanup test app");
}
