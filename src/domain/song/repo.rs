use super::model::{NewSong, Song};
use crate::domain::repository::{Connection, Transaction};
use crate::domain::shared::repository::{TimeCursor, TimePaginated};

pub trait Repo: Connection {
    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<Option<Song>, Box<dyn std::error::Error + Send + Sync>>;

    async fn find_by_keyword(
        &self,
        keyword: &str,
    ) -> Result<Vec<Song>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get songs ordered by creation time (newest first) with cursor pagination
    async fn find_by_time(
        &self,
        cursor: TimeCursor,
    ) -> Result<TimePaginated<Song>, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait TxRepo: Repo + Transaction
where
    Self::apply_update(..): Send,
{
    async fn create(
        &self,
        correction: &NewSong,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;

    async fn create_history(
        &self,
        correction: &NewSong,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;

    async fn apply_update(
        &self,
        correction: entity::correction::Model,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
