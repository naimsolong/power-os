// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use power_os_domain::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;
use validator::{Validate, ValidationError};

use crate::auth::extractor::AuthUser;
use crate::state::AppState;

const USER_ID_KEY: &str = "user_id";
const WORKSPACE_ID_KEY: &str = "workspace_id";

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
    pub name: Option<String>,
    pub workspace_name: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub user_id: UserId,
    pub workspace_id: WorkspaceId,
    pub email: String,
    pub name: Option<String>,
}

#[derive(Debug)]
pub enum AuthHandlerError {
    Validation(String),
    EmailTaken,
    InvalidCredentials,
    Internal,
}

impl IntoResponse for AuthHandlerError {
    fn into_response(self) -> Response {
        let (status, msg) = match self {
            AuthHandlerError::Validation(m) => (StatusCode::BAD_REQUEST, m),
            AuthHandlerError::EmailTaken => {
                (StatusCode::CONFLICT, "Email already registered".to_string())
            }
            AuthHandlerError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid credentials".to_string())
            }
            AuthHandlerError::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
        };
        (status, msg).into_response()
    }
}

pub async fn register(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AuthHandlerError> {
    payload
        .validate()
        .map_err(|e| AuthHandlerError::Validation(e.to_string()))?;

    let existing = sqlx::query("SELECT id FROM \"user\" WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    if existing.is_some() {
        return Err(AuthHandlerError::EmailTaken);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AuthHandlerError::Internal)?
        .to_string();

    let user_id = UserId::new();
    let workspace_id = WorkspaceId::new();
    let workspace_name = payload
        .workspace_name
        .unwrap_or_else(|| "My Workspace".to_string());

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    sqlx::query(
        r#"INSERT INTO "user" (id, email, name, password_hash) VALUES ($1, $2, $3, $4)"#,
    )
    .bind(user_id.0)
    .bind(&payload.email)
    .bind(&payload.name)
    .bind(&password_hash)
    .execute(&mut *tx)
    .await
    .map_err(|_| AuthHandlerError::Internal)?;

    sqlx::query("INSERT INTO workspace (id, name, slug) VALUES ($1, $2, $3)")
        .bind(workspace_id.0)
        .bind(&workspace_name)
        .bind(format!("ws-{}", workspace_id.0))
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    sqlx::query(
        "INSERT INTO workspace_user (workspace_id, user_id, role) VALUES ($1, $2, 'owner')",
    )
    .bind(workspace_id.0)
    .bind(user_id.0)
    .execute(&mut *tx)
    .await
    .map_err(|_| AuthHandlerError::Internal)?;

    tx.commit()
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    session
        .insert(USER_ID_KEY, user_id)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;
    session
        .insert(WORKSPACE_ID_KEY, workspace_id)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            user_id,
            workspace_id,
            email: payload.email,
            name: payload.name,
        }),
    ))
}

pub async fn login(
    State(state): State<AppState>,
    session: Session,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), AuthHandlerError> {
    payload
        .validate()
        .map_err(|e| AuthHandlerError::Validation(e.to_string()))?;

    let row = sqlx::query(
        r#"SELECT id, email, name, password_hash FROM "user" WHERE email = $1"#,
    )
    .bind(&payload.email)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AuthHandlerError::Internal)?
    .ok_or(AuthHandlerError::InvalidCredentials)?;

    let id: uuid::Uuid = row.try_get("id").map_err(|_| AuthHandlerError::Internal)?;
    let email: String = row.try_get("email").map_err(|_| AuthHandlerError::Internal)?;
    let name: Option<String> = row.try_get("name").map_err(|_| AuthHandlerError::Internal)?;
    let password_hash: String =
        row.try_get("password_hash").map_err(|_| AuthHandlerError::Internal)?;

    let parsed_hash =
        PasswordHash::new(&password_hash).map_err(|_| AuthHandlerError::Internal)?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthHandlerError::InvalidCredentials)?;

    let workspace_row = sqlx::query(
        "SELECT workspace_id FROM workspace_user WHERE user_id = $1 LIMIT 1",
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AuthHandlerError::Internal)?
    .ok_or(AuthHandlerError::InvalidCredentials)?;

    let workspace_id: uuid::Uuid = workspace_row
        .try_get("workspace_id")
        .map_err(|_| AuthHandlerError::Internal)?;

    let user_id = UserId::from(id);
    let workspace_id = WorkspaceId::from(workspace_id);

    session
        .insert(USER_ID_KEY, user_id)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;
    session
        .insert(WORKSPACE_ID_KEY, workspace_id)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            user_id,
            workspace_id,
            email,
            name,
        }),
    ))
}

pub async fn logout(session: Session) -> Result<StatusCode, AuthHandlerError> {
    session
        .delete()
        .await
        .map_err(|_| AuthHandlerError::Internal)?;
    Ok(StatusCode::OK)
}

pub async fn me(
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<AuthResponse>), AuthHandlerError> {
    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            user_id: auth_user.user_id,
            workspace_id: auth_user.workspace_id,
            email: auth_user.email,
            name: auth_user.name,
        }),
    ))
}

// This import is required by the `Validate` derive when custom validation is
// used; keep it available for future auth request validations.
#[allow(dead_code)]
type _ValidationError = ValidationError;
