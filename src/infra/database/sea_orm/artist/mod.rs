use std::collections::HashMap;
use std::path::PathBuf;

use entity::enums::{CorrectionStatus, EntityType};
use entity::sea_orm_active_enums::ArtistImageType;
use entity::{
    artist, artist_alias, artist_image, artist_link, artist_localized_name,
    artist_membership, artist_membership_role, artist_membership_tenure,
    correction, credit_role, image, language,
};
use itertools::{Itertools, izip};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait,
    FromQueryResult, LoaderTrait, QueryFilter, QueryOrder, QuerySelect, Select,
};
use sea_query::extension::postgres::PgBinOper;
use sea_query::{Cond, ExprTrait, Func, SimpleExpr};
use snafu::ResultExt;

use super::SeaOrmTxRepo;
use crate::domain::artist::model::{Artist, Membership, NewArtist, Tenure};
use crate::domain::artist::repo::{CommonFilter, FindManyFilter, Repo, TxRepo};
use crate::domain::credit_role::CreditRoleRef;
use crate::domain::repository::Connection;
use crate::domain::shared::model::{LocalizedName, Location};
use crate::domain::shared::repository::{TimeCursor, TimePaginated};

mod impls;

impl<T> Repo for T
where
    T: Connection,
    T::Conn: ConnectionTrait,
{
    async fn find_one(
        &self,
        id: i32,
        common: CommonFilter,
    ) -> Result<Option<Artist>, Box<dyn std::error::Error + Send + Sync>> {
        let select = artist::Entity::find()
            .filter(artist::Column::Id.eq(id))
            .filter(SimpleExpr::from(common));

        find_many_impl(select, self.conn())
            .await
            .map(|x| x.into_iter().next())
            .boxed()
    }

    async fn find_many(
        &self,
        filter: FindManyFilter,
        common: CommonFilter,
    ) -> Result<Vec<Artist>, Box<dyn std::error::Error + Send + Sync>> {
        let FindManyFilter::Keyword(keyword) = &filter;

        let search_term = Func::lower(keyword);

        let select = artist::Entity::find()
            .filter(
                Func::lower(artist::Column::Name.into_expr())
                    .binary(PgBinOper::Similarity, search_term.clone()),
            )
            .filter(SimpleExpr::from(common))
            .order_by_asc(
                Func::lower(artist::Column::Name.into_expr())
                    .binary(PgBinOper::SimilarityDistance, search_term),
            );

        find_many_impl(select, self.conn()).await.boxed()
    }

    async fn find_by_time(
        &self,
        cursor: TimeCursor,
        common: CommonFilter,
    ) -> Result<TimePaginated<Artist>, Box<dyn std::error::Error + Send + Sync>> {
        find_by_time_impl(cursor, common, EntityType::Artist, self.conn()).await.boxed()
    }
}

#[derive(FromQueryResult)]
struct ArtistImage {
    artist_id: i32,
    #[sea_orm(nested)]
    image: image::Model,
    r#type: ArtistImageType,
}

#[expect(clippy::too_many_lines, reason = "TODO")]
async fn find_many_impl(
    select: Select<artist::Entity>,
    db: &impl ConnectionTrait,
) -> Result<Vec<Artist>, DbErr> {
    let artists = select.all(db).await?;

    let ids = artists.iter().map(|x| x.id).unique().collect_vec();

    if ids.is_empty() {
        return Ok(vec![]);
    }

    let aliases = artist_alias::Entity::find()
        .filter(
            Condition::any()
                .add(artist_alias::Column::FirstId.is_in(ids.iter().copied()))
                .add(artist_alias::Column::SecondId.is_in(ids.iter().copied())),
        )
        .all(db)
        .await?;

    let artist_images = artist_image::Entity::find()
        .filter(artist_image::Column::ArtistId.is_in(ids.iter().copied()))
        .left_join(image::Entity)
        .into_model::<ArtistImage>()
        .all(db)
        .await?;

    let mut images_map: HashMap<i32, Vec<_>> = artist_images.into_iter().fold(
        HashMap::new(),
        |mut acc, artist_image| {
            acc.entry(artist_image.artist_id)
                .or_default()
                .push(artist_image);
            acc
        },
    );

    let (artists, images): (Vec<_>, Vec<_>) = artists
        .into_iter()
        .map(|artist| {
            let artist_images =
                images_map.remove(&artist.id).unwrap_or_default();
            (artist, artist_images)
        })
        .unzip();

    let links = artists.load_many(artist_link::Entity, db).await?;

    let localized_names =
        artists.load_many(artist_localized_name::Entity, db).await?;

    let artist_memberships = artist_membership::Entity::find()
        .filter(
            Condition::any()
                .add(
                    artist_membership::Column::MemberId
                        .is_in(ids.iter().copied()),
                )
                .add(artist_membership::Column::GroupId.is_in(ids)),
        )
        .all(db)
        .await?;

    let roles = artist_memberships
        .load_many_to_many(
            credit_role::Entity,
            artist_membership_role::Entity,
            db,
        )
        .await?;

    let join_leaves = artist_memberships
        .load_many(artist_membership_tenure::Entity, db)
        .await?;

    let group_association =
        izip!(artist_memberships, roles, join_leaves).collect_vec();

    let langs = language::Entity::find()
        .filter(
            language::Column::Id.is_in(
                localized_names
                    .iter()
                    .flat_map(|x| x.iter().map(|x| x.language_id)),
            ),
        )
        .all(db)
        .await?;

    let ret = izip!(artists, links, localized_names, images)
        .map(|(artist, links, localized_names, image)| {
            let start_date =
                match (artist.start_date, artist.start_date_precision) {
                    (Some(date), Some(precision)) => {
                        Some((date, precision).into())
                    }
                    _ => None,
                };

            let end_date = match (artist.end_date, artist.end_date_precision) {
                (Some(date), Some(precision)) => Some((date, precision).into()),
                _ => None,
            };

            let aliases = aliases
                .iter()
                .filter(|x| x.first_id == artist.id || x.second_id == artist.id)
                .map(|x| {
                    if x.first_id == artist.id {
                        x.second_id
                    } else {
                        x.first_id
                    }
                })
                .collect();

            let localized_names = localized_names
                .into_iter()
                .map(|model| LocalizedName {
                    name: model.name,
                    language: langs
                        .iter()
                        .find(|y| y.id == model.language_id)
                        .unwrap()
                        .clone()
                        .into(),
                })
                .collect();

            let memberships = group_association
                .iter()
                .filter(|(model, _, _)| {
                    if artist.artist_type.is_solo() {
                        model.member_id == artist.id
                    } else {
                        model.group_id == artist.id
                    }
                })
                .map(|(model, role, tenure)| {
                    let artist_id = if artist.artist_type.is_solo() {
                        model.group_id
                    } else {
                        model.member_id
                    };

                    let tenure = tenure
                        .iter()
                        .sorted_by_key(|x| x.id)
                        .map_into::<Tenure>()
                        .collect_vec();

                    Membership {
                        artist_id,
                        roles: role
                            .iter()
                            .map(|x| CreditRoleRef {
                                id: x.id,
                                name: x.name.clone(),
                            })
                            .collect_vec(),
                        tenure,
                    }
                })
                .collect();

            let profile_image_url = image
                .iter()
                .find(|x| x.r#type == ArtistImageType::Profile)
                .map(|x| {
                    let image = &x.image;
                    PathBuf::from_iter([&image.directory, &image.filename])
                        .to_string_lossy()
                        .to_string()
                });

            Artist {
                id: artist.id,
                name: artist.name,
                artist_type: artist.artist_type,
                text_aliases: artist.text_alias,
                start_date,
                end_date,

                aliases,
                links: links.into_iter().map(|x| x.url).collect_vec(),
                localized_names,
                start_location: Location {
                    country: artist.start_location_country,
                    province: artist.start_location_province,
                    city: artist.start_location_city,
                },
                current_location: Location {
                    country: artist.current_location_country,
                    province: artist.current_location_province,
                    city: artist.current_location_city,
                },
                memberships,
                profile_image_url,
                created_at: chrono::Utc::now(), // This will be updated by find_by_time_impl
            }
        })
        .collect_vec();

    Ok(ret)
}

async fn find_by_time_impl(
    cursor: TimeCursor,
    common: CommonFilter,
    entity_type: EntityType,
    db: &impl ConnectionTrait,
) -> Result<TimePaginated<Artist>, DbErr> {
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

    // Query artists by the entity IDs we found, applying common filter
    let select = artist::Entity::find()
        .filter(artist::Column::Id.is_in(entity_ids.clone()))
        .filter(SimpleExpr::from(common));
    let mut artists = find_many_impl(select, db).await?;

    // Update the created_at fields with the correction timestamps and sort by creation time
    for artist in &mut artists {
        if let Some(&created_at) = entity_times.get(&artist.id) {
            artist.created_at = created_at.to_utc();
        }
    }

    // Sort artists by creation time (newest first) to match the correction order
    artists.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Get next cursor from last item if we have results and hit the limit
    let next_cursor = if artists.len() == usize::from(cursor.limit) {
        artists.last().map(|artist| artist.created_at)
    } else {
        None
    };

    Ok(TimePaginated {
        items: artists,
        next_cursor,
    })
}

impl TxRepo for SeaOrmTxRepo {
    async fn create(
        &self,
        data: &NewArtist,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        Ok(impls::create_artist(data, self.conn()).await?.id)
    }

    async fn create_history(
        &self,
        data: &NewArtist,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        Ok(impls::create_artist_history(data, self.conn()).await?.id)
    }

    async fn apply_update(
        &self,
        correction: entity::correction::Model,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        impls::apply_update(correction, self.conn()).await.boxed()?;
        Ok(())
    }
}

impl From<CommonFilter> for SimpleExpr {
    fn from(value: CommonFilter) -> Self {
        Cond::all()
            .add_option(
                value
                    .artist_types
                    .map(|x| artist::Column::ArtistType.is_in(x)),
            )
            .add_option(value.exclusion.and_then(|x| {
                if x.is_empty() {
                    None
                } else {
                    Some(artist::Column::Id.is_not_in(x))
                }
            }))
            .into()
    }
}

#[cfg(test)]
mod tests {
    use sea_query::{PostgresQueryBuilder, QueryBuilder};

    use super::*;

    #[test]
    fn common_filter_into_expr() {
        let filter = CommonFilter {
            artist_types: None,
            exclusion: None,
        };

        let expr = SimpleExpr::from(filter);

        let mut str = String::new();
        PostgresQueryBuilder.prepare_simple_expr(&expr, &mut str);

        assert_eq!(str, "TRUE");
    }
}
