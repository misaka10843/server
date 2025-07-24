pub mod assertions;
pub mod client;
pub mod database;
pub mod fixtures;
pub mod test_utils;

use std::sync::Arc;

use axum::Router;
use axum_test::TestServer;
pub use client::TestClient;
pub use database::{TestDatabase, TestError};
pub use fixtures::TestFixtures;
pub use test_utils::*;

// Import the actual application modules from the main crate (for future use)
// use thcdb_rs::infra::state::AppState;
// use thcdb_rs::presentation::rest::state::ArcAppState;
// use thcdb_rs::infra::config::{Config, App, Email, EmailCreds, Middleware, LimitMiddleware};
// use thcdb_rs::infra::database::sea_orm::SeaOrmRepository;
// use lettre::transport::smtp::authentication::Credentials;
// use lettre::{AsyncSmtpTransport, Tokio1Executor};

/// Main test application that combines all test components
pub struct TestApp {
    pub database: TestDatabase,
    pub server: std::sync::Arc<TestServer>,
    pub client: TestClient,
    pub fixtures: TestFixtures,
}

impl TestApp {
    /// Create a new test application instance
    pub async fn new() -> Result<Self, TestError> {
        // Initialize test database
        let database = TestDatabase::new().await?;

        // For now, use the simple test router until we can properly set up the full app state
        // TODO: Create proper test app state and use the real router
        let app = create_simple_test_router();

        // Create test server
        let server =
            TestServer::new(app).map_err(|e| TestError::Http(e.to_string()))?;
        let server = std::sync::Arc::new(server);

        // Create test client
        let client = TestClient::new(server.clone());

        // Create test fixtures
        let fixtures = TestFixtures::new(database.connection().clone());

        Ok(Self {
            database,
            server: server.clone(),
            client,
            fixtures,
        })
    }

    /// Reset the test database to a clean state
    pub async fn reset(&self) -> Result<(), TestError> {
        self.database.reset().await
    }

    /// Explicitly stop the PostgreSQL instance and cleanup resources
    /// This ensures the database is stopped before the TestApp is dropped
    pub async fn stop(&mut self) -> Result<(), TestError> {
        self.database.stop().await
    }

    /// Cleanup all resources (alias for stop method for backward compatibility)
    pub async fn cleanup(&mut self) -> Result<(), TestError> {
        self.stop().await
    }
}

// Removed Drop trait implementation - all cleanup must be done manually
// This ensures deterministic cleanup and prevents resource leaks

// TODO: Implement proper test app state creation
// These functions are commented out until we can properly set up the full application state

// Create application state for testing
// async fn create_test_app_state(
// test_db: &TestDatabase,
// ) -> Result<ArcAppState, TestError> {
// Create a test configuration
// let config = create_test_config();
//
// Use the AppState::init method with the test config, but override the database connection
// let mut app_state = AppState::init(&config).await;
//
// Replace the database connection with our test database
// app_state.database = test_db.connection().clone();
// app_state.sea_orm_repo = SeaOrmRepository::new(test_db.connection().clone());
//
// Ok(ArcAppState::new(Arc::new(app_state)))
// }
//
// Create a test configuration
// fn create_test_config() -> Config {
// Config {
// database_url: "postgresql://test".to_string(), // Not used in tests
// redis_url: "redis://localhost:6379".to_string(), // Not used in tests
// app: App { port: 0 },
// email: Email {
// creds: EmailCreds {
// username: "test@example.com".to_string(),
// password: "test_password".to_string(),
// },
// host: "localhost".to_string(),
// },
// middleware: Middleware {
// limit: LimitMiddleware {
// req_per_sec: 100,
// burst_size: 10,
// },
// },
// }
// }
//
// Create a test Redis pool (mock implementation)
// async fn create_test_redis_pool() -> Result<fred::prelude::Pool, TestError> {
// For testing, we'll create a simple Redis pool that doesn't actually connect
// This is a minimal setup that satisfies the type requirements
// use fred::prelude::*;
//
// let mut config = Config::from_url("redis://localhost:6379").map_err(|e| TestError::Setup {
// message: format!("Failed to create Redis config: {}", e)
// })?;
// config.fail_fast = false; // Don't fail fast in tests
//
// let pool = Pool::new(config, None, None, None, 1).map_err(|e| TestError::Setup {
// message: format!("Failed to create Redis pool: {}", e)
// })?;
//
// Don't actually initialize/connect for tests - just return the pool
// Ok(pool)
// }
//
// Create a test email transport
// fn create_test_email_transport(config: &Config) -> AsyncSmtpTransport<Tokio1Executor> {
// let creds = Credentials::new(
// config.email.creds.username.clone(),
// config.email.creds.password.clone(),
// );
//
// Create a test transport that doesn't actually send emails
// AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous("localhost")
// .credentials(creds)
// .build()
// }

/// Create a simple test router for basic testing
fn create_simple_test_router() -> Router {
    use axum::http::StatusCode;
    use axum::routing::get;

    Router::new()
        .route("/health_check", get(|| async { StatusCode::OK }))
        .route("/", get(|| async { "Welcome to test server" }))
    // Add more basic routes as needed for testing
}
