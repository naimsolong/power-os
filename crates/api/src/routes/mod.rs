// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, Router};

use crate::auth;
use crate::state::AppState;

mod health;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .nest("/api/auth", auth::router())
        .with_state(state)
}
