//! Common test utilities and helper functions
//!
//! This module provides reusable utilities for integration tests,
//! reducing code duplication and improving test maintainability.

use std::process::Command;

use tokio::time::{Duration, sleep};

/// Count PostgreSQL processes on the system
///
/// This function is used to monitor for resource leaks in tests.
/// It counts all postgres processes while excluding test processes.
pub fn count_postgres_processes() -> usize {
    let output = Command::new("ps").args(&["aux"]).output();

    match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);

            // Count lines containing "postgres" but exclude test processes
            stdout
                .lines()
                .filter(|line| {
                    line.contains("postgres")
                        && !line.contains("test")
                        && !line.contains("ps aux")
                })
                .count()
        }
        Err(_) => {
            // If ps command fails, return 0 (can't monitor on this system)
            0
        }
    }
}

/// Wait for a short duration to allow cleanup operations to complete
pub async fn wait_for_cleanup() {
    sleep(Duration::from_millis(500)).await;
}

/// Wait for a longer duration to allow multiple cleanup operations
pub async fn wait_for_full_cleanup() {
    sleep(Duration::from_secs(1)).await;
}

/// Assert that process count growth is within acceptable limits
///
/// This helps verify that tests are not leaking PostgreSQL processes.
///
/// # Arguments
/// * `baseline` - The initial process count before test operations
/// * `current` - The current process count after test operations
/// * `max_growth` - Maximum acceptable growth in process count
///
/// # Panics
/// Panics if the process growth exceeds the maximum acceptable limit
pub fn assert_process_growth_acceptable(
    baseline: usize,
    current: usize,
    max_growth: usize,
) {
    let growth = current.saturating_sub(baseline);

    if growth > max_growth {
        panic!(
            "Resource leak detected! Process growth: {} (acceptable: ≤ {})",
            growth, max_growth
        );
    }
}

/// Verify that two values are different (useful for isolation tests)
pub fn assert_isolated<T: PartialEq + std::fmt::Debug>(
    value1: &T,
    value2: &T,
    message: &str,
) {
    assert_ne!(value1, value2, "{}", message);
}

/// Verify that a value is positive (useful for ID validation)
pub fn assert_valid_id(id: i32, entity_type: &str) {
    assert!(
        id > 0,
        "{} should have a valid ID, got: {}",
        entity_type,
        id
    );
}

/// Verify that a string is not empty
pub fn assert_non_empty_string(value: &str, field_name: &str) {
    assert!(!value.is_empty(), "{} should not be empty", field_name);
}

/// Test configuration constants
pub mod config {
    /// Maximum acceptable process growth for resource leak tests
    pub const MAX_PROCESS_GROWTH: usize = 2;

    /// Number of iterations for stress tests
    pub const STRESS_TEST_ITERATIONS: usize = 10;

    /// Number of apps to create in batch tests
    pub const BATCH_SIZE: usize = 3;
}

/// Common test patterns and workflows
pub mod patterns {
    use super::super::TestError;
    use super::*;
    use crate::TestApp;

    /// Create a test app, use it briefly, then cleanup
    /// Returns the created user for verification
    pub async fn create_use_cleanup_app()
    -> Result<entity::user::Model, TestError> {
        let mut app = TestApp::new().await?;

        let user = app.fixtures.create_user(None).await?;

        app.cleanup().await?;

        Ok(user)
    }

    /// Test basic HTTP endpoints for an app
    pub async fn test_basic_endpoints(app: &TestApp) -> Result<(), TestError> {
        use axum::http::StatusCode;

        // Test health check
        let health_response = app.client.get("/health_check").await;
        assert_eq!(health_response.status_code(), StatusCode::OK);

        // Test home endpoint
        let home_response = app.client.get("/").await;
        assert_eq!(home_response.status_code(), StatusCode::OK);

        // Test 404 handling
        let not_found_response = app.client.get("/nonexistent").await;
        assert_eq!(not_found_response.status_code(), StatusCode::NOT_FOUND);

        Ok(())
    }

    /// Create test entities and verify they exist in the database
    pub async fn create_and_verify_entities(
        app: &TestApp,
    ) -> Result<(), TestError> {
        use entity::*;
        use sea_orm::{EntityTrait, PaginatorTrait};

        // Create entities
        let user = app.fixtures.create_user(None).await?;
        let artist = app.fixtures.create_artist(None).await?;

        // Verify in database
        let db = app.database.connection();

        let found_user = user::Entity::find_by_id(user.id)
            .one(db)
            .await?
            .ok_or_else(|| TestError::Setup {
                message: "User not found".to_string(),
            })?;

        assert_eq!(found_user.name, user.name);

        let found_artist = artist::Entity::find_by_id(artist.id)
            .one(db)
            .await?
            .ok_or_else(|| TestError::Setup {
                message: "Artist not found".to_string(),
            })?;

        assert_eq!(found_artist.name, artist.name);

        // Verify counts
        let user_count = user::Entity::find().count(db).await?;

        assert!(user_count >= 1, "Should have at least 1 user");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assert_valid_id() {
        assert_valid_id(1, "User");
        assert_valid_id(999, "Artist");
    }

    #[test]
    #[should_panic(expected = "User should have a valid ID")]
    fn test_assert_valid_id_panics_on_zero() {
        assert_valid_id(0, "User");
    }

    #[test]
    #[should_panic(expected = "User should have a valid ID")]
    fn test_assert_valid_id_panics_on_negative() {
        assert_valid_id(-1, "User");
    }

    #[test]
    fn test_assert_non_empty_string() {
        assert_non_empty_string("test", "name");
        assert_non_empty_string("a", "title");
    }

    #[test]
    #[should_panic(expected = "name should not be empty")]
    fn test_assert_non_empty_string_panics() {
        assert_non_empty_string("", "name");
    }
}
