// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    routing::{delete, get, patch, post},
    Router,
};

use crate::state::AppState;

mod extractor;
mod handlers;

pub use extractor::{AuthUser, AuthenticatedUserId, AuthenticatedWorkspaceId, WorkspaceRole};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/logout", post(handlers::logout))
        .route("/me", get(handlers::me))
        .route("/forgot-password", post(handlers::forgot_password))
        .route("/reset-password", post(handlers::reset_password))
        .route("/workspace/users", get(handlers::list_workspace_users))
        .route(
            "/workspace/users/{user_id}/role",
            patch(handlers::update_workspace_user_role),
        )
        .route(
            "/workspace/users/{user_id}",
            delete(handlers::remove_workspace_user),
        )
}
