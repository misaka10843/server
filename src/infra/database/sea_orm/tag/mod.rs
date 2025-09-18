use entity::enums::{CorrectionStatus, EntityType};
use entity::tag::Column::Name;
use entity::{
    correction, tag, tag_alternative_name, tag_alternative_name_history, tag_history,
    tag_relation, tag_relation_history,
};
use itertools::izip;
use sea_orm::ActiveValue::{NotSet, Set};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseTransaction, DbErr,
    EntityTrait, IntoActiveValue, LoaderTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use sea_query::extension::postgres::PgBinOper::*;
use sea_query::{ExprTrait, Func};
use snafu::ResultExt;

use crate::domain::repository::Connection;
use crate::domain::shared::repository::{TimeCursor, TimePaginated};
use crate::domain::tag::model::{
    AlternativeName, NewTag, NewTagRelation, Tag, TagRelation,
};
use crate::domain::tag::{Repo, TxRepo};

mod impls;
use impls::*;

impl<T> Repo for T
where
    T: Connection,
    T::Conn: ConnectionTrait,
{
    async fn find_by_id(
        &self,
        id: i32,
    ) -> Result<Option<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        let select = tag::Entity::find().filter(tag::Column::Id.eq(id));
        find_many_impl(select, self.conn())
            .await
            .map(|x| x.into_iter().next())
            .boxed()
    }

    async fn find_by_keyword(
        &self,
        keyword: &str,
    ) -> Result<Vec<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        let search_term = Func::lower(keyword);

        let select = tag::Entity::find()
            .filter(
                Func::lower(Name.into_expr())
                    .binary(Similarity, search_term.clone()),
            )
            .order_by_asc(
                Func::lower(Name.into_expr())
                    .binary(SimilarityDistance, search_term),
            );
        find_many_impl(select, self.conn()).await.boxed()
    }

    async fn find_by_time(
        &self,
        cursor: TimeCursor,
    ) -> Result<TimePaginated<Tag>, Box<dyn std::error::Error + Send + Sync>> {
        find_by_time_impl(cursor, EntityType::Tag, self.conn()).await.boxed()
    }
}

async fn find_many_impl(
    select: sea_orm::Select<tag::Entity>,
    db: &impl ConnectionTrait,
) -> Result<Vec<Tag>, DbErr> {
    let tags = select.all(db).await?;

    let alt_names = tags.load_many(tag_alternative_name::Entity, db).await?;

    let relations = tag_relation::Entity::find()
        .filter(
            tag_relation::Column::TagId.is_in(tags.iter().map(|tag| tag.id)),
        )
        .all(db)
        .await?;

    Ok(izip!(tags, alt_names)
        .map(|(tag, alt_names)| Tag {
            id: tag.id,
            name: tag.name,
            r#type: tag.r#type,
            short_description: Some(tag.short_description),
            description: Some(tag.description),
            alt_names: alt_names
                .into_iter()
                .map(|an| AlternativeName {
                    id: an.id,
                    name: an.name,
                })
                .collect(),
            relations: relations
                .iter()
                .filter(|relation| relation.tag_id == tag.id)
                .map(|r| TagRelation {
                    related_tag_id: r.related_tag_id,
                    r#type: r.r#type,
                })
                .collect(),
            created_at: chrono::Utc::now(), // This will be updated by find_by_time_impl
        })
        .collect())
}

async fn find_by_time_impl(
    cursor: TimeCursor,
    entity_type: EntityType,
    db: &impl ConnectionTrait,
) -> Result<TimePaginated<Tag>, DbErr> {
    use std::collections::HashMap;

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

    // Query tags by the entity IDs we found
    let select = tag::Entity::find().filter(tag::Column::Id.is_in(entity_ids.clone()));
    let mut tags = find_many_impl(select, db).await?;

    // Update the created_at fields with the correction timestamps and sort by creation time
    for tag in &mut tags {
        if let Some(&created_at) = entity_times.get(&tag.id) {
            tag.created_at = created_at.to_utc();
        }
    }

    // Sort tags by creation time (newest first) to match the correction order
    tags.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Get next cursor from last item if we have results and hit the limit
    let next_cursor = if tags.len() == usize::from(cursor.limit) {
        tags.last().map(|tag| tag.created_at)
    } else {
        None
    };

    Ok(TimePaginated {
        items: tags,
        next_cursor,
    })
}

impl TxRepo for crate::infra::database::sea_orm::SeaOrmTxRepo {
    async fn create(
        &self,
        data: &NewTag,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let tag = create_tag_impl(data, self.conn()).await?;

        Ok(tag.id)
    }

    async fn create_history(
        &self,
        data: &NewTag,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        create_history_impl(data, self.conn())
            .await
            .map(|x| x.id)
            .boxed()
    }

    async fn apply_update(
        &self,
        correction: entity::correction::Model,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        apply_correction(correction, self.conn()).await.boxed()
    }
}

async fn create_tag_impl(
    data: &NewTag,
    tx: &DatabaseTransaction,
) -> Result<tag::Model, DbErr> {
    let tag_model = tag::ActiveModel {
        id: NotSet,
        name: data.name.to_string().into_active_value(),
        r#type: Set(data.r#type),
        short_description: data
            .short_description
            .clone()
            .unwrap_or_default()
            .into_active_value(),
        description: data
            .description
            .clone()
            .unwrap_or_default()
            .into_active_value(),
    };

    let tag = tag_model.insert(tx).await?;

    if let Some(alt_names) = &data.alt_names {
        create_alt_name(tag.id, alt_names, tx).await?;
    }

    if let Some(relations) = &data.relations {
        create_relation(tag.id, relations, tx).await?;
    }

    Ok(tag)
}

async fn create_history_impl(
    data: &NewTag,
    tx: &DatabaseTransaction,
) -> Result<tag_history::Model, DbErr> {
    let history_model = tag_history::ActiveModel {
        id: NotSet,
        name: data.name.to_string().into_active_value(),
        r#type: Set(data.r#type),
        short_description: data
            .short_description
            .clone()
            .unwrap_or_default()
            .into_active_value(),
        description: data
            .description
            .clone()
            .unwrap_or_default()
            .into_active_value(),
    };

    let history = history_model.insert(tx).await?;

    if let Some(alt_names) = &data.alt_names {
        create_alt_name_history(history.id, alt_names, tx).await?;
    }

    if let Some(relations) = &data.relations {
        create_relation_history(history.id, relations, tx).await?;
    }

    Ok(history)
}

async fn create_alt_name(
    tag_id: i32,
    alt_names: &[String],
    tx: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if alt_names.is_empty() {
        return Ok(());
    }

    let active_models =
        alt_names
            .iter()
            .map(|name| tag_alternative_name::ActiveModel {
                id: NotSet,
                tag_id: Set(tag_id),
                name: Set(name.clone()),
                is_origin_language: Set(false),
                language_id: Set(None),
            });

    tag_alternative_name::Entity::insert_many(active_models)
        .exec(tx)
        .await?;

    Ok(())
}

async fn create_alt_name_history(
    history_id: i32,
    alt_names: &[String],
    tx: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if alt_names.is_empty() {
        return Ok(());
    }

    let active_models = alt_names.iter().map(|name| {
        tag_alternative_name_history::ActiveModel {
            id: NotSet,
            history_id: Set(history_id),
            name: Set(name.clone()),
            is_origin_language: Set(false),
            language_id: Set(None),
        }
    });

    tag_alternative_name_history::Entity::insert_many(active_models)
        .exec(tx)
        .await?;

    Ok(())
}

async fn create_relation(
    tag_id: i32,
    relations: &[NewTagRelation],
    tx: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if relations.is_empty() {
        return Ok(());
    }

    let active_models =
        relations.iter().map(|relation| tag_relation::ActiveModel {
            tag_id: Set(tag_id),
            related_tag_id: Set(relation.related_tag_id),
            r#type: Set(relation.r#type),
        });

    tag_relation::Entity::insert_many(active_models)
        .exec(tx)
        .await?;

    Ok(())
}

async fn create_relation_history(
    history_id: i32,
    relations: &[NewTagRelation],
    tx: &DatabaseTransaction,
) -> Result<(), DbErr> {
    if relations.is_empty() {
        return Ok(());
    }

    let active_models =
        relations
            .iter()
            .map(|relation| tag_relation_history::ActiveModel {
                history_id: Set(history_id),
                related_tag_id: Set(relation.related_tag_id),
                r#type: Set(relation.r#type),
            });

    tag_relation_history::Entity::insert_many(active_models)
        .exec(tx)
        .await?;

    Ok(())
}
