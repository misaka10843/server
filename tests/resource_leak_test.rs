//! Test to verify that PostgreSQL resource leaks are fixed
//!
//! This test creates multiple TestApp instances and verifies that
//! PostgreSQL processes are properly cleaned up after each test.

#[path = "integration/lib.rs"]
mod integration;

use integration::config::{
    BATCH_SIZE, MAX_PROCESS_GROWTH, STRESS_TEST_ITERATIONS,
};
use integration::{
    TestApp, assert_process_growth_acceptable, count_postgres_processes,
    wait_for_cleanup, wait_for_full_cleanup,
};

#[tokio::test]
async fn test_no_resource_leak_with_explicit_cleanup() {
    let baseline_count = count_postgres_processes();

    // Create and properly cleanup multiple test apps
    for i in 1..=STRESS_TEST_ITERATIONS {
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

        wait_for_cleanup().await;
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
async fn test_rapid_creation_and_cleanup() {
    let baseline_count = count_postgres_processes();

    // Create and cleanup apps rapidly in batches
    for _batch in 1..=BATCH_SIZE {
        let mut apps = Vec::new();

        // Create apps rapidly
        for i in 1..=BATCH_SIZE {
            let app = TestApp::new()
                .await
                .expect(&format!("Failed to create app {}", i));
            apps.push(app);
        }

        // Use them briefly
        for app in apps.iter() {
            let user = app
                .fixtures
                .create_user(None)
                .await
                .expect("Failed to create user");
            assert!(!user.name.is_empty(), "User should have a name");
        }

        // Cleanup them all
        for mut app in apps.into_iter() {
            app.cleanup().await.expect("Failed to cleanup app");
        }

        wait_for_cleanup().await;
    }

    wait_for_full_cleanup().await;
    let final_count = count_postgres_processes();
    assert_process_growth_acceptable(
        baseline_count,
        final_count,
        MAX_PROCESS_GROWTH,
    );
}
