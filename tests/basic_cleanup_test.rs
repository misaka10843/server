//! Basic test to verify that PostgreSQL cleanup is working
//!
//! This test creates a few TestApp instances sequentially and verifies that
//! PostgreSQL processes are properly cleaned up.

// Import only the necessary modules without importing the API tests
mod integration {
    pub mod common;

    pub use common::{
        TestApp, TestDatabase, TestError, TestFixtures, assert_isolated,
        assert_non_empty_string, assert_process_growth_acceptable,
        assert_valid_id, config, count_postgres_processes, fixtures, patterns,
        wait_for_cleanup, wait_for_full_cleanup,
    };
}

use integration::config::MAX_PROCESS_GROWTH;
use integration::{
    TestApp, assert_non_empty_string, assert_process_growth_acceptable,
    assert_valid_id, count_postgres_processes, patterns, wait_for_cleanup,
    wait_for_full_cleanup,
};

#[tokio::test]
async fn test_basic_cleanup_works() {
    let baseline_count = count_postgres_processes();

    // Test 1: Create and cleanup a single app
    {
        let mut app = TestApp::new().await.expect("Failed to create app 1");

        let user = app
            .fixtures
            .create_user(None)
            .await
            .expect("Failed to create user in app 1");
        assert!(!user.name.is_empty(), "User should have a name");

        app.cleanup().await.expect("Failed to cleanup app 1");
    }

    wait_for_full_cleanup().await;

    // Test 2: Create and cleanup another app
    {
        let mut app = TestApp::new().await.expect("Failed to create app 2");

        let user = app
            .fixtures
            .create_user(None)
            .await
            .expect("Failed to create user in app 2");
        assert!(!user.name.is_empty(), "User should have a name");

        app.cleanup().await.expect("Failed to cleanup app 2");
    }

    wait_for_full_cleanup().await;

    let final_count = count_postgres_processes();
    assert_process_growth_acceptable(
        baseline_count,
        final_count,
        MAX_PROCESS_GROWTH,
    );
}

#[tokio::test]
async fn test_multiple_sequential_cleanups() {
    let baseline_count = count_postgres_processes();

    // Test multiple sequential app creations and cleanups
    for i in 1..=3 {
        let mut app = TestApp::new()
            .await
            .expect(&format!("Failed to create app {}", i));
        let user = app
            .fixtures
            .create_user(None)
            .await
            .expect(&format!("Failed to create user in app {}", i));
        assert!(!user.name.is_empty(), "User should have a name");

        app.cleanup()
            .await
            .expect(&format!("Failed to cleanup app {}", i));

        // Wait between iterations to ensure cleanup completes
        wait_for_cleanup().await;
    }

    // Final wait and check
    wait_for_full_cleanup().await;
    let final_count = count_postgres_processes();

    // Verify no resource leaks after multiple sequential operations
    assert_process_growth_acceptable(
        baseline_count,
        final_count,
        MAX_PROCESS_GROWTH,
    );
}

#[tokio::test]
async fn test_explicit_stop_with_operations() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    let user = app
        .fixtures
        .create_user(None)
        .await
        .expect("Failed to create user");

    assert_non_empty_string(&user.name, "User name");
    assert_valid_id(user.id, "User");

    patterns::test_basic_endpoints(&app)
        .await
        .expect("HTTP endpoints test failed");

    app.stop().await.expect("Failed to stop PostgreSQL");
}

#[tokio::test]
async fn test_database_operations_before_stop() {
    let mut app = TestApp::new().await.expect("Failed to create test app");

    patterns::create_and_verify_entities(&app)
        .await
        .expect("Entity creation and verification failed");

    app.stop().await.expect("Failed to stop database");
}
