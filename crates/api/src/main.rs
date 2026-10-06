// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, Router};
use std::net::SocketAddr;
use tracing::info;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = routes::router();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    info!("Power OS API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
