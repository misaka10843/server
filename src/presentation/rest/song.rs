use axum::Json;
use axum::extract::{Path, Query, State};
use libfp::BifunctorExt;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

use super::extract::CurrentUser;
use super::state::{
    ArcAppState, {self},
};
use crate::application::correction::NewCorrectionDto;
use crate::application::song::{CreateError, UpsertCorrectionError};
use crate::domain::shared::repository::{TimeCursor, TimePaginated};
use crate::domain::song::model::{NewSong, Song};
use crate::infra::error::Error;
use crate::presentation::api_response::{Data, Message};

const TAG: &str = "Song";

pub fn router() -> OpenApiRouter<ArcAppState> {
    OpenApiRouter::new()
        .routes(routes!(create_song))
        .routes(routes!(update_song))
        .routes(routes!(find_song_by_id))
        .routes(routes!(find_song_by_keyword))
        .routes(routes!(find_songs_by_time))
}

super::data! {
    DataOptionSong, Option<Song>
    DataVecSong, Vec<Song>
    DataTimePaginatedSong, TimePaginated<Song>
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/song/{id}",
    responses(
		(status = 200, body = DataOptionSong),
		Error
    ),
)]
async fn find_song_by_id(
    State(service): State<state::SongService>,
    Path(id): Path<i32>,
) -> Result<Data<Option<Song>>, Error> {
    service.find_by_id(id).await.bimap_into()
}

#[derive(Deserialize, ToSchema, IntoParams)]
struct KwQuery {
    keyword: String,
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/song",
    params(KwQuery),
    responses(
		(status = 200, body = DataVecSong),
		Error
    ),
)]
async fn find_song_by_keyword(
    State(service): State<state::SongService>,
    Query(query): Query<KwQuery>,
) -> Result<Data<Vec<Song>>, Error> {
    service.find_by_keyword(&query.keyword).await.bimap_into()
}

#[utoipa::path(
    get,
    tag = TAG,
    path = "/song/recent",
    params(TimeCursor),
    responses(
		(status = 200, body = DataTimePaginatedSong),
		Error
    ),
)]
async fn find_songs_by_time(
    State(service): State<state::SongService>,
    Query(cursor): Query<TimeCursor>,
) -> Result<Data<TimePaginated<Song>>, Error> {
    service.find_by_time(cursor).await.bimap_into()
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/song",
    request_body = NewCorrectionDto<NewSong>,
    responses(
		(status = 200, body = Message),
        (status = 401),
        CreateError
    ),
)]
async fn create_song(
    CurrentUser(user): CurrentUser,
    State(service): State<state::SongService>,
    Json(dto): Json<NewCorrectionDto<NewSong>>,
) -> Result<Message, CreateError> {
    service.create(dto.with_author(user)).await?;

    Ok(Message::ok())
}

#[utoipa::path(
    post,
    tag = TAG,
    path = "/song/{id}",
    request_body = NewCorrectionDto<NewSong>,
    responses(
		(status = 200, body = Message),
        (status = 401),
        UpsertCorrectionError
    ),
)]
async fn update_song(
    CurrentUser(user): CurrentUser,
    State(service): State<state::SongService>,
    Path(song_id): Path<i32>,
    Json(input): Json<NewCorrectionDto<NewSong>>,
) -> Result<Message, UpsertCorrectionError> {
    service
        .upsert_correction(song_id, input.with_author(user))
        .await?;

    Ok(Message::ok())
}
