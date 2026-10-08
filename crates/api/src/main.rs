// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use std::env;
use std::net::SocketAddr;

use sqlx::PgPool;
use time::Duration;
use tower_sessions::{cookie::SameSite, Expiry, SessionManagerLayer};
use tower_sessions_sqlx_store::PostgresStore;
use tracing::info;

mod auth;
mod routes;
mod state;

use state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let cookie_secure = env::var("COOKIE_SECURE")
        .map(|v| v.parse().unwrap_or(true))
        .unwrap_or(true);

    let pool = PgPool::connect(&database_url).await?;

    let session_store = PostgresStore::new(pool.clone());
    session_store.migrate().await?;

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(cookie_secure)
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)));

    let state = AppState::new(pool);
    let app = routes::router(state).layer(session_layer);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Power OS API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
