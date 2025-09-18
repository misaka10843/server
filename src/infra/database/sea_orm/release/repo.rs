use entity::{correction, release};
use entity::enums::{CorrectionStatus, EntityType};
use sea_orm::{
    ColumnTrait, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter,
    QueryOrder, QuerySelect,
};
use snafu::ResultExt;
use std::collections::HashMap;

use super::impls::*;
use crate::domain::release::model::Release;
use crate::domain::release::repo::{Filter, Repo};
use crate::domain::repository::Connection;
use crate::domain::shared::repository::{TimeCursor, TimePaginated};

impl<T> Repo for T
where
    T: Connection,
    T::Conn: ConnectionTrait,
{
    async fn find_one(
        &self,
        filter: Filter,
    ) -> Result<Option<Release>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(find_many_impl(filter.into_select().limit(1), self.conn())
            .await?
            .into_iter()
            .next())
    }

    async fn find_many(
        &self,
        // TODO: many filter
        filter: Filter,
    ) -> Result<Vec<Release>, Box<dyn std::error::Error + Send + Sync>> {
        find_many_impl(filter.into_select(), self.conn())
            .await
            .boxed()
    }

    async fn exist(
        &self,
        id: i32,
    ) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        Ok(release::Entity::find()
            .select_only()
            .expr(1)
            .filter(release::Column::Id.eq(id))
            .count(self.conn())
            .await
            .boxed()?
            > 0)
    }

    async fn find_by_time(
        &self,
        cursor: TimeCursor,
    ) -> Result<TimePaginated<Release>, Box<dyn std::error::Error + Send + Sync>> {
        find_by_time_impl(cursor, EntityType::Release, self.conn()).await.boxed()
    }
}

async fn find_by_time_impl(
    cursor: TimeCursor,
    entity_type: EntityType,
    db: &impl ConnectionTrait,
) -> Result<TimePaginated<Release>, sea_orm::DbErr> {
    // Query corrections to find entity creation times, ordered by newest first
    let mut correction_query = correction::Entity::find()
        .filter(correction::Column::EntityType.eq(entity_type))
        .filter(correction::Column::Status.eq(CorrectionStatus::Approved))
        .order_by_desc(correction::Column::CreatedAt)
        .limit(u64::from(cursor.limit));

    // Apply cursor filter if provided
    if let Some(after) = cursor.after {
        correction_query = correction_query.filter(correction::Column::CreatedAt.lt(after));
    }

    let corrections = correction_query.all(db).await?;
    
    if corrections.is_empty() {
        return Ok(TimePaginated {
            items: vec![],
            next_cursor: None,
        });
    }

    // Extract entity IDs and their creation times
    let entity_ids: Vec<i32> = corrections.iter().map(|c| c.entity_id).collect();
    let entity_times: HashMap<i32, chrono::DateTime<chrono::FixedOffset>> = corrections
        .iter()
        .map(|c| (c.entity_id, c.created_at))
        .collect();

    // Query releases by the entity IDs we found
    let select = release::Entity::find().filter(release::Column::Id.is_in(entity_ids.clone()));
    let mut releases = find_many_impl(select, db).await?;

    // Update the created_at fields with the correction timestamps and sort by creation time
    for release in &mut releases {
        if let Some(&created_at) = entity_times.get(&release.id) {
            release.created_at = created_at.to_utc();
        }
    }

    // Sort releases by creation time (newest first) to match the correction order
    releases.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Get next cursor from last item if we have results and hit the limit
    let next_cursor = if releases.len() == usize::from(cursor.limit) {
        releases.last().map(|release| release.created_at)
    } else {
        None
    };

    Ok(TimePaginated {
        items: releases,
        next_cursor,
    })
}
