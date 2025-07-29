use postgresql_embedded::PostgreSQL;
use sea_orm::DatabaseConnection;
use thcdb_rs::infra::database::get_connection;

pub struct TestDatabase {
    instance: postgresql_embedded::PostgreSQL,
    connection: DatabaseConnection,
}

const DATABASE_NAME: &str = "thcdb_test";

impl TestDatabase {
    pub async fn new() -> anyhow::Result<Self> {
        let mut postgresql = postgresql_embedded::PostgreSQL::default();
        postgresql.setup().await?;
        postgresql.start().await?;

        let connection = init(&mut postgresql).await?;

        Ok(Self {
            instance: postgresql,
            connection,
        })
    }

    pub async fn reset(&mut self) -> anyhow::Result<()> {
        self.instance.drop_database(DATABASE_NAME).await?;
        init(&mut self.instance).await?;
        Ok(())
    }

    pub async fn stop(self) -> anyhow::Result<()> {
        self.instance.stop().await?;
        Ok(())
    }

    pub fn connection(&self) -> &DatabaseConnection {
        &self.connection
    }
}

async fn init(inst: &mut PostgreSQL) -> anyhow::Result<DatabaseConnection> {
    inst.create_database(DATABASE_NAME).await?;
    inst.database_exists(DATABASE_NAME).await?;
    inst.drop_database(DATABASE_NAME).await?;

    let url = inst.settings().url(DATABASE_NAME);

    let conn = get_connection(&url).await;

    Ok(conn)
}
