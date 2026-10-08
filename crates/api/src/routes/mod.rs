// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, Router};

use crate::auth;
use crate::state::AppState;

mod deal_stages;
mod deals;
mod error;
mod health;
mod parties;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .nest("/api/auth", auth::router())
        .nest("/api/parties", parties::router())
        .nest("/api/deal-stages", deal_stages::router())
        .nest("/api/deals", deals::router())
        .with_state(state)
}
