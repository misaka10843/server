use super::model::{NewTag, Tag};
use crate::domain::repository::{Connection, Transaction};
use crate::domain::shared::repository::pagination::{TimeCursor, TimePaginated};

pub trait Repo: Connection {
    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<Option<Tag>, Box<dyn std::error::Error + Send + Sync>>;

    async fn find_by_keyword(
        &self,
        keyword: &str,
    ) -> Result<Vec<Tag>, Box<dyn std::error::Error + Send + Sync>>;

    /// Get tags ordered by creation time (newest first) with cursor pagination
    async fn find_by_time(
        &self,
        cursor: TimeCursor,
    ) -> Result<TimePaginated<Tag>, Box<dyn std::error::Error + Send + Sync>>;
}

pub trait TxRepo: Repo + Transaction
where
    Self::apply_update(..): Send,
{
    async fn create(
        &self,
        correction: &NewTag,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;

    async fn create_history(
        &self,
        correction: &NewTag,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>>;

    async fn apply_update(
        &self,
        correction: entity::correction::Model,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
