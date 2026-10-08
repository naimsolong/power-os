// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    routing::{get, post},
    Router,
};

use crate::state::AppState;

mod extractor;
mod handlers;

pub use extractor::{AuthUser, AuthenticatedUserId, AuthenticatedWorkspaceId};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
}
