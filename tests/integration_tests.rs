//! Integration tests entry point
//!
//! This file serves as the main entry point for integration tests.
//! It includes all the integration test modules and provides a way to run them with `cargo test`.

#[path = "integration/lib.rs"]
mod integration;

// Re-export the integration test modules for easy access
pub use integration::*;

// Example of how to run a specific test:
// cargo test test_health_check
//
// Run all integration tests:
// cargo test --test integration_tests
//
// Run tests with output:
// cargo test --test integration_tests -- --nocapture
//
// Run a specific test module:
// cargo test --test integration_tests api::health
