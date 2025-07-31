use std::sync::Arc;

use thcdb_rs::infra::logger::Logger;
use thcdb_rs::infra::singleton::APP_CONFIG;
use thcdb_rs::infra::state::AppState;
use thcdb_rs::infra::worker::Worker;

#[cfg(all(feature = "release", unix))]
mod alloc {
    use tikv_jemallocator::Jemalloc;
    #[global_allocator]
    static GLOBAL: Jemalloc = Jemalloc;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv()?;

    Logger::init();

    tracing::info!("Starting server");

    let state = AppState::init(&APP_CONFIG).await;

    Worker {
        redis_pool: state.redis_pool(),
    }
    .init();

    let listener = tokio::net::TcpListener::bind(format!(
        "0.0.0.0:{}",
        APP_CONFIG.app.port
    ))
    .await?;

    tracing::info!(
        "Server listening on http://127.0.0.1:{}",
        APP_CONFIG.app.port
    );

    thcdb_rs::presentation::rest::listen(listener, Arc::new(state)).await?;

    Ok(())
}
