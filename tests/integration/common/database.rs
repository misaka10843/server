use std::path::PathBuf;

use anyhow::{Context, Result};
use postgresql_embedded::{PostgreSQL, Settings};
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use sea_orm_migration::MigratorTrait;
use thcdb_rs::infra::database::sea_orm::enum_table::sync_enum_table;
const DB_NAME: &str = "thc_test";

pub struct TestDatabase {
    postgres: PostgreSQL,
    connection: DatabaseConnection,
}

impl TestDatabase {
    pub async fn new() -> Result<Self> {
        dotenvy::dotenv().ok();

        let settings = Settings::default();

        let mut postgres = PostgreSQL::new(settings);
        postgres.setup().await?;
        postgres.start().await?;
        postgres.create_database(DB_NAME).await?;

        let database_url = postgres.settings().url(DB_NAME);

        let connection = create_connection(&database_url).await?;

        migration::Migrator::up(&connection, None).await?;

        sync_enum_table(&connection).await?;

        Ok(Self {
            postgres,
            connection,
        })
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }

    pub fn url(&self) -> String {
        self.postgres.settings().url(DB_NAME)
    }

    pub async fn stop(self) -> Result<()> {
        self.postgres
            .stop()
            .await
            .context("PostgreSQL instance stopped successfully")
    }
}

impl Drop for TestDatabase {
    fn drop(&mut self) {
        let settings = self.postgres.settings();
        let _ = std::fs::remove_dir_all(&settings.data_dir);
        let _ = settings.password_file.parent().map(std::fs::remove_dir_all);
    }
}

async fn create_connection(url: &str) -> Result<DatabaseConnection> {
    let opt = ConnectOptions::new(url)
        .sqlx_logging(false)
        .min_connections(1)
        .max_connections(1)
        .to_owned();

    let conn = Database::connect(opt).await?;
    Ok(conn)
}
