//! Integration tests for the Touhou Cloud DB application
//!
//! This module provides comprehensive integration testing capabilities using:
//! - `postgresql_embedded` for isolated test databases
//! - `axum_test` for HTTP API testing
//! - Custom test utilities and fixtures

pub mod api;
pub mod common;
pub mod database;

// Re-export commonly used types for convenience
// Re-export sub-modules for easier access
pub use common::{
    TestApp, TestDatabase, TestError, TestFixtures, assert_isolated,
    assert_non_empty_string, assert_process_growth_acceptable, assert_valid_id,
    assertions, config, count_postgres_processes, fixtures, patterns,
    wait_for_cleanup, wait_for_full_cleanup,
};
