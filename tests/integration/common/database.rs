use postgresql_embedded::{PostgreSQL, Settings};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use thcdb_rs::infra::database::sea_orm::enum_table::sync_enum_table;
use thiserror::Error;

/// Custom error type for test operations
#[derive(Debug, Error)]
pub enum TestError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("PostgreSQL embedded error: {0}")]
    PostgresEmbedded(#[from] postgresql_embedded::Error),

    #[error("HTTP client error: {0}")]
    Http(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Test setup error: {message}")]
    Setup { message: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

// Note: axum_test error handling will be implemented based on the actual API

/// Test database manager using postgresql_embedded
pub struct TestDatabase {
    postgres: PostgreSQL,
    connection: DatabaseConnection,
    database_url: String,
}

impl TestDatabase {
    /// Create a new test database instance
    pub async fn new() -> Result<Self, TestError> {
        dotenvy::dotenv().ok();
        // Clean up any leftover temporary PostgreSQL directories before starting
        Self::cleanup_old_temp_dirs();

        // Configure embedded PostgreSQL
        let settings = Settings {
            // Use a temporary directory for the database
            temporary: true,
            // Set a custom port to avoid conflicts
            port: 0, // Let the system choose an available port
            ..Default::default()
        };

        // Start the embedded PostgreSQL instance
        let mut postgres = PostgreSQL::new(settings);
        postgres.setup().await?;
        postgres.start().await?;

        // Get the database URL
        let database_url = postgres.settings().url("postgres");

        // Create database connection
        let connection = create_connection(&database_url).await?;

        // Run migrations
        migration::Migrator::up(&connection, None).await?;

        sync_enum_table(&connection).await?;

        Ok(Self {
            postgres,
            connection,
            database_url,
        })
    }

    /// Get a reference to the database connection
    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    /// Get the database URL
    pub fn url(&self) -> &str {
        &self.database_url
    }

    /// Reset the database to a clean state
    pub async fn reset(&self) -> Result<(), TestError> {
        // For now, we'll truncate all tables
        // In a more sophisticated setup, we might use transactions or recreate the database
        self.truncate_all_tables().await?;

        // Re-sync enum tables after truncation
        // Note: This requires access to the main crate's enum_table module
        // For now, we'll skip this in tests and handle it manually if needed

        // Force a connection refresh to clear any cached data
        self.connection.ping().await?;

        Ok(())
    }

    /// Explicitly stop the PostgreSQL instance
    /// This should be called when you want to ensure the database is stopped
    /// before the TestDatabase is dropped
    pub async fn stop(&mut self) -> Result<(), TestError> {
        // First, close the database connection to avoid connection issues during shutdown
        // Note: We can't actually close the connection here because it doesn't have a close method
        // But we can drop our reference to it

        // Stop the PostgreSQL instance
        match self.postgres.stop().await {
            Ok(_) => {
                eprintln!("PostgreSQL instance stopped successfully");
                Ok(())
            }
            Err(e) => {
                let error_msg = e.to_string();
                eprintln!(
                    "Failed to stop PostgreSQL gracefully: {}",
                    error_msg
                );

                // Check if the error indicates the server is already stopped or stopping
                if error_msg.contains("does not exist")
                    || error_msg.contains("Is server running?")
                    || error_msg.contains("waiting for server to shut down")
                    || error_msg.contains("already stopped")
                {
                    eprintln!(
                        "PostgreSQL instance appears to be already stopped or stopping"
                    );
                    return Ok(());
                }

                // For other errors, try a more aggressive approach
                eprintln!("Attempting force cleanup of PostgreSQL processes");

                // Try gentle termination first
                let _ = std::process::Command::new("pkill")
                    .args(&["-TERM", "-f", "postgres"])
                    .output();

                // Wait a bit for graceful shutdown
                tokio::time::sleep(tokio::time::Duration::from_millis(500))
                    .await;

                // Then force kill if needed
                let _ = std::process::Command::new("pkill")
                    .args(&["-9", "-f", "postgres"])
                    .output();

                // Don't return an error - we've done our best to clean up
                eprintln!("Force cleanup completed");
                Ok(())
            }
        }
    }

    /// Truncate all tables in the database
    async fn truncate_all_tables(&self) -> Result<(), TestError> {
        // For simplicity in tests, we'll use a simpler approach
        // Disable foreign key checks temporarily
        self.connection
            .execute_unprepared("SET session_replication_role = replica;")
            .await?;

        // Truncate main tables (add more as needed)
        let tables = [
            "user_list_item",
            "user_list",
            "user_following",
            "song_credit",
            "song_artist",
            "release_track",
            "release_artist",
            "artist_membership",
            "group_member_join_leave",
            "group_member",
            "correction",
            "comment",
            "song_lyrics",
            "song",
            "release",
            "artist",
            "tag",
            "event",
            "label",
            "image",
            "user",
        ];

        for table in tables {
            let truncate_query = format!(
                "TRUNCATE TABLE \"{}\" RESTART IDENTITY CASCADE;",
                table
            );
            match self.connection.execute_unprepared(&truncate_query).await {
                Ok(_) => eprintln!("Successfully truncated table: {}", table),
                Err(e) => {
                    // Ignore errors for tables that might not exist
                    eprintln!(
                        "Skipping table {} (may not exist): {}",
                        table, e
                    );
                }
            }
        }

        // Re-enable foreign key checks
        self.connection
            .execute_unprepared("SET session_replication_role = DEFAULT;")
            .await?;

        Ok(())
    }

    /// Clean up old temporary PostgreSQL directories to prevent disk space issues
    fn cleanup_old_temp_dirs() {
        // Clean up temporary directories older than 1 hour
        let cleanup_commands = [
            // Remove old temporary PostgreSQL directories
            (
                "find",
                vec![
                    "/tmp", "-name", ".tmp*", "-type", "d", "-mmin", "+60",
                    "-exec", "rm", "-rf", "{}", "+",
                ],
            ),
            // Also clean up any postgres processes that might be hanging
            ("pkill", vec!["-f", "postgres.*tmp"]),
        ];

        for (cmd, args) in cleanup_commands {
            let _ = std::process::Command::new(cmd).args(&args).output();
        }

        eprintln!("Cleaned up old temporary PostgreSQL directories");
    }
}

// Removed Drop trait implementation - all cleanup must be done manually
// This ensures deterministic cleanup and prevents resource leaks

/// Create a database connection with test-specific settings
async fn create_connection(url: &str) -> Result<DatabaseConnection, TestError> {
    let opt = ConnectOptions::new(url)
        .sqlx_logging(false) // Disable SQL logging in tests for cleaner output
        .min_connections(1)
        .max_connections(5) // Limit connections for tests
        .to_owned();

    let conn = Database::connect(opt).await?;
    Ok(conn)
}
