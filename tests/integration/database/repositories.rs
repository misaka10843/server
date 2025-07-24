use entity::*;
use sea_orm::{ActiveModelTrait, EntityTrait, Set};

use crate::integration::{TestDatabase, TestFixtures};

#[tokio::test]
async fn test_user_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let user = fixtures
        .create_user(None)
        .await
        .expect("Failed to create user");

    assert!(user.id > 0, "User should have a valid ID");
    assert!(!user.name.is_empty(), "User should have a name");

    // Read
    let found_user = user::Entity::find_by_id(user.id)
        .one(connection)
        .await
        .expect("Failed to find user")
        .expect("User should exist");

    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.name, user.name);

    // Update
    let mut user_update: user::ActiveModel = found_user.into();
    user_update.bio = Set(Some("Updated bio".to_string()));

    let updated_user = user_update
        .update(connection)
        .await
        .expect("Failed to update user");

    assert_eq!(updated_user.bio, Some("Updated bio".to_string()));

    // Delete
    let delete_result = user::Entity::delete_by_id(updated_user.id)
        .exec(connection)
        .await
        .expect("Failed to delete user");

    assert_eq!(delete_result.rows_affected, 1);

    // Verify deletion
    let deleted_user = user::Entity::find_by_id(updated_user.id)
        .one(connection)
        .await
        .expect("Failed to query for deleted user");

    assert!(deleted_user.is_none(), "User should be deleted");
}

#[tokio::test]
async fn test_artist_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let artist = fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    assert!(artist.id > 0, "Artist should have a valid ID");
    assert!(!artist.name.is_empty(), "Artist should have a name");

    // Read
    let found_artist = artist::Entity::find_by_id(artist.id)
        .one(connection)
        .await
        .expect("Failed to find artist")
        .expect("Artist should exist");

    assert_eq!(found_artist.id, artist.id);
    assert_eq!(found_artist.name, artist.name);

    // Update
    let mut artist_update: artist::ActiveModel = found_artist.into();
    artist_update.current_location_country =
        Set(Some("Updated Country".to_string()));

    let updated_artist = artist_update
        .update(connection)
        .await
        .expect("Failed to update artist");

    assert_eq!(
        updated_artist.current_location_country,
        Some("Updated Country".to_string())
    );

    // Delete
    let delete_result = artist::Entity::delete_by_id(updated_artist.id)
        .exec(connection)
        .await
        .expect("Failed to delete artist");

    assert_eq!(delete_result.rows_affected, 1);
}

#[tokio::test]
async fn test_song_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let song = fixtures
        .create_song(None)
        .await
        .expect("Failed to create song");

    assert!(song.id > 0, "Song should have a valid ID");
    assert!(!song.title.is_empty(), "Song should have a title");

    // Read
    let found_song = song::Entity::find_by_id(song.id)
        .one(connection)
        .await
        .expect("Failed to find song")
        .expect("Song should exist");

    assert_eq!(found_song.id, song.id);
    assert_eq!(found_song.title, song.title);

    // Update
    let mut song_update: song::ActiveModel = found_song.into();
    song_update.title = Set("Updated Song Title".to_string());

    let updated_song = song_update
        .update(connection)
        .await
        .expect("Failed to update song");

    assert_eq!(updated_song.title, "Updated Song Title");

    // Delete
    let delete_result = song::Entity::delete_by_id(updated_song.id)
        .exec(connection)
        .await
        .expect("Failed to delete song");

    assert_eq!(delete_result.rows_affected, 1);
}

#[tokio::test]
async fn test_release_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let release = fixtures
        .create_release(None)
        .await
        .expect("Failed to create release");

    assert!(release.id > 0, "Release should have a valid ID");
    assert!(!release.title.is_empty(), "Release should have a title");

    // Read
    let found_release = release::Entity::find_by_id(release.id)
        .one(connection)
        .await
        .expect("Failed to find release")
        .expect("Release should exist");

    assert_eq!(found_release.id, release.id);
    assert_eq!(found_release.title, release.title);

    // Update
    let mut release_update: release::ActiveModel = found_release.into();
    release_update.title = Set("Updated Release Title".to_string());

    let updated_release = release_update
        .update(connection)
        .await
        .expect("Failed to update release");

    assert_eq!(updated_release.title, "Updated Release Title");

    // Delete
    let delete_result = release::Entity::delete_by_id(updated_release.id)
        .exec(connection)
        .await
        .expect("Failed to delete release");

    assert_eq!(delete_result.rows_affected, 1);
}

#[tokio::test]
async fn test_tag_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let tag = fixtures
        .create_tag(None)
        .await
        .expect("Failed to create tag");

    assert!(tag.id > 0, "Tag should have a valid ID");
    assert!(!tag.name.is_empty(), "Tag should have a name");

    // Read
    let found_tag = tag::Entity::find_by_id(tag.id)
        .one(connection)
        .await
        .expect("Failed to find tag")
        .expect("Tag should exist");

    assert_eq!(found_tag.id, tag.id);
    assert_eq!(found_tag.name, tag.name);

    // Update
    let mut tag_update: tag::ActiveModel = found_tag.into();
    tag_update.description = Set("Updated description".to_string());

    let updated_tag = tag_update
        .update(connection)
        .await
        .expect("Failed to update tag");

    assert_eq!(updated_tag.description, "Updated description");

    // Delete
    let delete_result = tag::Entity::delete_by_id(updated_tag.id)
        .exec(connection)
        .await
        .expect("Failed to delete tag");

    assert_eq!(delete_result.rows_affected, 1);
}

#[tokio::test]
async fn test_event_repository_crud() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create
    let event = fixtures
        .create_event(None)
        .await
        .expect("Failed to create event");

    assert!(event.id > 0, "Event should have a valid ID");
    assert!(!event.name.is_empty(), "Event should have a name");

    // Read
    let found_event = event::Entity::find_by_id(event.id)
        .one(connection)
        .await
        .expect("Failed to find event")
        .expect("Event should exist");

    assert_eq!(found_event.id, event.id);
    assert_eq!(found_event.name, event.name);

    // Update
    let mut event_update: event::ActiveModel = found_event.into();
    event_update.description = Set("Updated description".to_string());

    let updated_event = event_update
        .update(connection)
        .await
        .expect("Failed to update event");

    assert_eq!(updated_event.description, "Updated description");

    // Delete
    let delete_result = event::Entity::delete_by_id(updated_event.id)
        .exec(connection)
        .await
        .expect("Failed to delete event");

    assert_eq!(delete_result.rows_affected, 1);
}

#[tokio::test]
async fn test_repository_isolation() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let fixtures = TestFixtures::new(test_db.connection().clone());
    let connection = test_db.connection();

    // Create entities in one test
    let user = fixtures
        .create_user(None)
        .await
        .expect("Failed to create user");
    let artist = fixtures
        .create_artist(None)
        .await
        .expect("Failed to create artist");

    // Verify they exist
    let user_exists = user::Entity::find_by_id(user.id)
        .one(connection)
        .await
        .expect("Failed to query user");
    assert!(user_exists.is_some(), "User should exist");

    let artist_exists = artist::Entity::find_by_id(artist.id)
        .one(connection)
        .await
        .expect("Failed to query artist");
    assert!(artist_exists.is_some(), "Artist should exist");

    // Reset database
    test_db.reset().await.expect("Failed to reset database");

    // Verify they no longer exist
    let user_exists = user::Entity::find_by_id(user.id)
        .one(connection)
        .await
        .expect("Failed to query user");
    assert!(user_exists.is_none(), "User should not exist after reset");

    let artist_exists = artist::Entity::find_by_id(artist.id)
        .one(connection)
        .await
        .expect("Failed to query artist");
    assert!(
        artist_exists.is_none(),
        "Artist should not exist after reset"
    );
}
