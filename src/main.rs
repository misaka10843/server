mod api_response;
mod controller;
mod error;
mod middleware;
mod model;
mod resolver;
mod service;

use crate::service::user::AuthSession;
use axum::extract::FromRef;
use axum::{routing::get, Router};
use axum_login::tower_sessions::cookie::time::Duration;
use axum_login::tower_sessions::{Expiry, SessionManagerLayer};
use axum_login::AuthManagerLayerBuilder;
use sea_orm::DatabaseConnection;
use service::{
    database::get_db_connection, ReleaseService, SongService, UserService,
};
use tokio::signal;
use tower_sessions_redis_store::RedisStore;
use tracing_subscriber::fmt::time::ChronoLocal;
use utoipa_swagger_ui::SwaggerUi;

#[derive(Clone, FromRef)]
pub struct AppState {
    database: DatabaseConnection,
    user_service: UserService,
    song_service: SongService,
    release_service: ReleaseService,
    image_service: service::image::Service,
}

impl AppState {
    pub async fn init(url: &str) -> Self {
        let database = get_db_connection(url).await;

        Self {
            database: database.clone(),
            user_service: UserService::new(database.clone()),
            song_service: SongService::new(database.clone()),
            release_service: ReleaseService::new(database.clone()),
            image_service: service::image::Service::new(database.clone()),
        }
    }
}

#[tokio::main]
async fn main() {
    tracing::info!("Starting server");

    dotenvy::dotenv().unwrap();

    tracing_subscriber::fmt()
        .with_timer(ChronoLocal::new("%Y-%m-%d %H:%M:%S".to_string()))
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .init();

    let config = service::config::Service::init();
    let state = AppState::init(&config.database_url).await;

    let redis_service = service::redis::Service::init(&config).await;

    let pool = redis_service.pool();

    let session_store = RedisStore::new(pool);
    let session_layer = SessionManagerLayer::new(session_store)
        .with_http_only(true)
        .with_expiry(Expiry::OnInactivity(Duration::days(30)));

    let auth_layer =
        AuthManagerLayerBuilder::new(state.user_service.clone(), session_layer)
            .build();

    let (api_router, api) = controller::api_router().split_for_parts();

    let doc_router = api_router
        .merge(SwaggerUi::new("/docs").url("/openapi.json", api.clone()));

    let router = Router::new()
        .route(
            "/",
            get(|session: AuthSession| async {
                if let Some(user) = session.user {
                    format!("Hello, {}!", user.name)
                } else {
                    "Hello, world!".into()
                }
            }),
        )
        .merge(doc_router)
        .layer(auth_layer)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        config.server_port
    ))
    .await
    .unwrap();

    tracing::info!(
        "Server listening on http://127.0.0.1:{}",
        config.server_port
    );

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            match signal::ctrl_c().await {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("Unable to listen for shutdown signal: {}", err);
                }
            }

            redis_service.quit().await.unwrap()
        })
        .await
        .unwrap();
}
