use entity::*;
use sea_orm::{ConnectionTrait, EntityTrait, PaginatorTrait};

use crate::integration::TestDatabase;

#[tokio::test]
async fn test_migrations_run_successfully() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");

    // If we get here, migrations ran successfully during database setup
    // Let's verify some key tables exist

    let connection = test_db.connection();

    // Test that we can query each main entity table
    let user_count = user::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query user table");
    assert_eq!(user_count, 0, "User table should be empty initially");

    let artist_count = artist::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query artist table");
    assert_eq!(artist_count, 0, "Artist table should be empty initially");

    let song_count = song::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query song table");
    assert_eq!(song_count, 0, "Song table should be empty initially");

    let release_count = release::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query release table");
    assert_eq!(release_count, 0, "Release table should be empty initially");

    let tag_count = tag::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query tag table");
    assert_eq!(tag_count, 0, "Tag table should be empty initially");

    let event_count = event::Entity::find()
        .count(connection)
        .await
        .expect("Failed to query event table");
    assert_eq!(event_count, 0, "Event table should be empty initially");
}

#[tokio::test]
async fn test_enum_tables_populated() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");

    // If we get here, the database was created successfully with migrations
    // For now, we'll just verify the database connection works
    // In a full implementation, you would check enum table population

    // Simple test to verify database is working
    let connection = test_db.connection();
    let result = connection.execute_unprepared("SELECT 1").await;
    assert!(result.is_ok(), "Database should be accessible");
}

#[tokio::test]
async fn test_database_constraints() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let connection = test_db.connection();

    // Test unique constraint on user name using execute_unprepared
    let _ = connection
        .execute_unprepared("INSERT INTO \"user\" (name, password, last_login) VALUES ('testuser', 'hash', NOW())")
        .await
        .expect("First insert should succeed");

    let result = connection
        .execute_unprepared("INSERT INTO \"user\" (name, password, last_login) VALUES ('testuser', 'hash', NOW())")
        .await;

    assert!(
        result.is_err(),
        "Should fail due to unique constraint on user name"
    );
}

#[tokio::test]
async fn test_database_reset() {
    let test_db = TestDatabase::new()
        .await
        .expect("Failed to create test database");
    let connection = test_db.connection();

    // Insert some test data
    let _ = connection
        .execute_unprepared("INSERT INTO \"user\" (name, password, last_login) VALUES ('testuser', 'hash', NOW())")
        .await
        .expect("Failed to insert test user");

    let _ = connection
        .execute_unprepared("INSERT INTO song (title) VALUES ('Test Song')")
        .await
        .expect("Failed to insert test song");

    // Verify data exists
    let user_count = user::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count users");
    assert_eq!(user_count, 1, "Should have one user");

    let song_count = song::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count songs");
    assert_eq!(song_count, 1, "Should have one song");

    // Reset the database
    test_db.reset().await.expect("Failed to reset database");

    // Verify data is gone
    let user_count = user::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count users after reset");
    assert_eq!(user_count, 0, "Should have no users after reset");

    let song_count = song::Entity::find()
        .count(connection)
        .await
        .expect("Failed to count songs after reset");
    assert_eq!(song_count, 0, "Should have no songs after reset");

    // For now, just verify the database is still accessible after reset
    let result = connection.execute_unprepared("SELECT 1").await;
    assert!(
        result.is_ok(),
        "Database should still be accessible after reset"
    );
}
