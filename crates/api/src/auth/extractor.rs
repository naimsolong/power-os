// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use power_os_domain::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;

use crate::state::AppState;

const USER_ID_KEY: &str = "user_id";
const WORKSPACE_ID_KEY: &str = "workspace_id";

/// Authenticated user populated from the session by [`AuthUser`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: UserId,
    pub workspace_id: WorkspaceId,
    pub email: String,
    pub name: Option<String>,
}

/// Extension value inserted by the [`AuthUser`] extractor so downstream
/// handlers and middleware can read the authenticated IDs directly.
#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedUserId(pub UserId);

/// Extension value inserted by the [`AuthUser`] extractor so downstream
/// handlers and middleware can read the active workspace directly.
#[derive(Clone, Copy, Debug)]
pub struct AuthenticatedWorkspaceId(pub WorkspaceId);

#[derive(Debug)]
pub enum AuthError {
    MissingSession,
    InvalidSession,
    UserNotFound,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AuthError::MissingSession => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            AuthError::InvalidSession => (StatusCode::UNAUTHORIZED, "Invalid session"),
            AuthError::UserNotFound => (StatusCode::UNAUTHORIZED, "User not found"),
        };
        (status, message).into_response()
    }
}

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let session = parts
            .extensions
            .get::<Session>()
            .ok_or(AuthError::MissingSession)?
            .clone();

        let user_id: UserId = session
            .get(USER_ID_KEY)
            .await
            .map_err(|_| AuthError::InvalidSession)?
            .ok_or(AuthError::InvalidSession)?;

        let workspace_id: WorkspaceId = session
            .get(WORKSPACE_ID_KEY)
            .await
            .map_err(|_| AuthError::InvalidSession)?
            .ok_or(AuthError::InvalidSession)?;

        let row = sqlx::query(
            r#"
            SELECT u.email, u.name
            FROM "user" u
            JOIN workspace_user wu ON wu.user_id = u.id
            WHERE u.id = $1 AND wu.workspace_id = $2
            "#,
        )
        .bind(user_id.0)
        .bind(workspace_id.0)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AuthError::InvalidSession)?
        .ok_or(AuthError::UserNotFound)?;

        let email: String = row.try_get("email").map_err(|_| AuthError::InvalidSession)?;
        let name: Option<String> = row.try_get("name").map_err(|_| AuthError::InvalidSession)?;

        parts.extensions.insert(AuthenticatedUserId(user_id));
        parts.extensions.insert(AuthenticatedWorkspaceId(workspace_id));

        Ok(AuthUser {
            user_id,
            workspace_id,
            email,
            name,
        })
    }
}
