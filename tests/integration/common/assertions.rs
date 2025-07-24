use axum::http::StatusCode;
use axum_test::TestResponse;
use serde_json::Value;

/// Custom assertions for test responses
pub trait ResponseAssertions {
    /// Assert that the response has a specific status code
    fn assert_status(&self, expected_status: StatusCode) -> &Self;

    /// Assert that the response is successful (2xx)
    fn assert_success(&self) -> &Self;

    /// Assert that the response is a client error (4xx)
    fn assert_client_error(&self) -> &Self;

    /// Assert that the response is a server error (5xx)
    fn assert_server_error(&self) -> &Self;

    /// Assert that the response contains valid JSON
    fn assert_json_valid(&self) -> &Self;

    /// Assert that the response JSON contains a specific field
    fn assert_json_has_field(&self, field: &str) -> &Self;

    /// Assert that the response JSON field has a specific value
    fn assert_json_field_equals(&self, field: &str, expected: &Value) -> &Self;
}

impl ResponseAssertions for TestResponse {
    fn assert_status(&self, expected_status: StatusCode) -> &Self {
        let actual_status = self.status_code();
        assert_eq!(
            actual_status,
            expected_status,
            "Expected status {}, but got {}. Response body: {}",
            expected_status,
            actual_status,
            self.text()
        );
        self
    }

    fn assert_success(&self) -> &Self {
        let status = self.status_code();
        assert!(
            status.is_success(),
            "Expected successful status (2xx), but got {}. Response body: {}",
            status.as_u16(),
            self.text()
        );
        self
    }

    fn assert_client_error(&self) -> &Self {
        let status = self.status_code();
        assert!(
            status.is_client_error(),
            "Expected client error status (4xx), but got {}. Response body: {}",
            status.as_u16(),
            self.text()
        );
        self
    }

    fn assert_server_error(&self) -> &Self {
        let status = self.status_code();
        assert!(
            status.is_server_error(),
            "Expected server error status (5xx), but got {}. Response body: {}",
            status.as_u16(),
            self.text()
        );
        self
    }

    fn assert_json_valid(&self) -> &Self {
        // Try to parse as JSON to verify it's valid
        let _: Value = self.json();
        self
    }

    fn assert_json_has_field(&self, field: &str) -> &Self {
        let json: Value = self.json();
        assert!(
            json.get(field).is_some(),
            "Expected JSON to contain field '{}', but it was not found. JSON: {}",
            field,
            json
        );
        self
    }

    fn assert_json_field_equals(&self, field: &str, expected: &Value) -> &Self {
        let json: Value = self.json();
        let actual = json.get(field).unwrap_or_else(|| {
            panic!(
                "Expected JSON to contain field '{}', but it was not found. JSON: {}",
                field, json
            )
        });

        assert_eq!(
            actual, expected,
            "Expected field '{}' to equal {:?}, but got {:?}. Full JSON: {}",
            field, expected, actual, json
        );
        self
    }
}

/// Helper macros for common assertions
#[macro_export]
macro_rules! assert_json_eq {
    ($response:expr, $expected:expr) => {
        let actual: serde_json::Value = $response.json();
        let expected: serde_json::Value = $expected;
        assert_eq!(
            actual, expected,
            "JSON response does not match expected value"
        );
    };
}

#[macro_export]
macro_rules! assert_json_contains {
    ($response:expr, $field:expr, $expected:expr) => {
        let json: serde_json::Value = $response.json();
        let actual = json.get($field).unwrap_or_else(|| {
            panic!("Field '{}' not found in JSON response: {}", $field, json)
        });
        assert_eq!(
            actual, &$expected,
            "Field '{}' does not match expected value",
            $field
        );
    };
}

/// Database assertion helpers
pub mod db {
    use entity::*;
    use sea_orm::{DatabaseConnection, EntityTrait};

    /// Assert that a user exists in the database
    pub async fn assert_user_exists(db: &DatabaseConnection, user_id: i32) {
        let user = user::Entity::find_by_id(user_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(user.is_some(), "User with ID {} should exist", user_id);
    }

    /// Assert that an artist exists in the database
    pub async fn assert_artist_exists(db: &DatabaseConnection, artist_id: i32) {
        let artist = artist::Entity::find_by_id(artist_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(
            artist.is_some(),
            "Artist with ID {} should exist",
            artist_id
        );
    }

    /// Assert that a song exists in the database
    pub async fn assert_song_exists(db: &DatabaseConnection, song_id: i32) {
        let song = song::Entity::find_by_id(song_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(song.is_some(), "Song with ID {} should exist", song_id);
    }

    /// Assert that a release exists in the database
    pub async fn assert_release_exists(
        db: &DatabaseConnection,
        release_id: i32,
    ) {
        let release = release::Entity::find_by_id(release_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(
            release.is_some(),
            "Release with ID {} should exist",
            release_id
        );
    }

    /// Assert that a tag exists in the database
    pub async fn assert_tag_exists(db: &DatabaseConnection, tag_id: i32) {
        let tag = tag::Entity::find_by_id(tag_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(tag.is_some(), "Tag with ID {} should exist", tag_id);
    }

    /// Assert that an event exists in the database
    pub async fn assert_event_exists(db: &DatabaseConnection, event_id: i32) {
        let event = event::Entity::find_by_id(event_id)
            .one(db)
            .await
            .expect("Database query failed");

        assert!(event.is_some(), "Event with ID {} should exist", event_id);
    }
}
