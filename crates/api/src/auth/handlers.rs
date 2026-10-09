// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use power_os_domain::{UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tower_sessions::Session;
use tracing::{error, info};
use validator::{Validate, ValidationError};

use crate::auth::extractor::{AuthUser, WorkspaceRole};
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
    pub role: String,
}

#[derive(Debug)]
pub enum AuthHandlerError {
    Validation(String),
    EmailTaken,
    InvalidCredentials,
    NotFound,
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
            AuthHandlerError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AuthHandlerError::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Internal error".to_string(),
            ),
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

    sqlx::query(r#"INSERT INTO "user" (id, email, name, password_hash) VALUES ($1, $2, $3, $4)"#)
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

    tx.commit().await.map_err(|_| AuthHandlerError::Internal)?;

    crate::routes::fiscal_periods::seed_fiscal_year(&state.db, workspace_id, user_id)
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
            role: WorkspaceRole::Owner.as_str().to_string(),
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

    let row = sqlx::query(r#"SELECT id, email, name, password_hash FROM "user" WHERE email = $1"#)
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?
        .ok_or(AuthHandlerError::InvalidCredentials)?;

    let id: uuid::Uuid = row.try_get("id").map_err(|_| AuthHandlerError::Internal)?;
    let email: String = row
        .try_get("email")
        .map_err(|_| AuthHandlerError::Internal)?;
    let name: Option<String> = row
        .try_get("name")
        .map_err(|_| AuthHandlerError::Internal)?;
    let password_hash: String = row
        .try_get("password_hash")
        .map_err(|_| AuthHandlerError::Internal)?;

    let parsed_hash = PasswordHash::new(&password_hash).map_err(|_| AuthHandlerError::Internal)?;
    Argon2::default()
        .verify_password(payload.password.as_bytes(), &parsed_hash)
        .map_err(|_| AuthHandlerError::InvalidCredentials)?;

    let workspace_row =
        sqlx::query("SELECT workspace_id, role FROM workspace_user WHERE user_id = $1 LIMIT 1")
            .bind(id)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| AuthHandlerError::Internal)?
            .ok_or(AuthHandlerError::InvalidCredentials)?;

    let workspace_id: uuid::Uuid = workspace_row
        .try_get("workspace_id")
        .map_err(|_| AuthHandlerError::Internal)?;
    let role: String = workspace_row
        .try_get("role")
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
            role,
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

pub async fn me(auth_user: AuthUser) -> Result<(StatusCode, Json<AuthResponse>), AuthHandlerError> {
    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            user_id: auth_user.user_id,
            workspace_id: auth_user.workspace_id,
            email: auth_user.email,
            name: auth_user.name,
            role: auth_user.role.as_str().to_string(),
        }),
    ))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ForgotPasswordRequest {
    #[validate(email(message = "Invalid email"))]
    pub email: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct ResetPasswordRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters"))]
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceUserResponse {
    pub user_id: UserId,
    pub email: String,
    pub name: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRoleRequest {
    pub role: WorkspaceRole,
}

pub async fn forgot_password(
    State(state): State<AppState>,
    Json(payload): Json<ForgotPasswordRequest>,
) -> Result<(StatusCode, Json<PasswordResetResponse>), AuthHandlerError> {
    payload
        .validate()
        .map_err(|e| AuthHandlerError::Validation(e.to_string()))?;

    let user = sqlx::query(r#"SELECT id FROM "user" WHERE email = $1"#)
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    if let Some(user) = user {
        let user_id: uuid::Uuid = user.try_get("id").map_err(|_| AuthHandlerError::Internal)?;
        let token = uuid::Uuid::new_v4().to_string();
        let expires_at = time::OffsetDateTime::now_utc() + time::Duration::hours(1);

        sqlx::query(
            r#"
            INSERT INTO password_reset_token (user_id, token, expires_at)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(user_id)
        .bind(&token)
        .bind(expires_at)
        .execute(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

        let reset_url = format!(
            "{}?token={}",
            std::env::var("PASSWORD_RESET_URL")
                .unwrap_or_else(|_| "http://localhost:5173/reset-password".to_string()),
            token
        );
        info!(email = %payload.email, "password reset token generated: {}", reset_url);
    }

    Ok((
        StatusCode::OK,
        Json(PasswordResetResponse {
            message: "If an account exists, a reset link has been sent.".to_string(),
        }),
    ))
}

pub async fn reset_password(
    State(state): State<AppState>,
    Json(payload): Json<ResetPasswordRequest>,
) -> Result<StatusCode, AuthHandlerError> {
    payload
        .validate()
        .map_err(|e| AuthHandlerError::Validation(e.to_string()))?;

    let token_row = sqlx::query(
        r#"
        SELECT id, user_id, expires_at, used_at
        FROM password_reset_token
        WHERE token = $1
        "#,
    )
    .bind(&payload.token)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AuthHandlerError::Internal)?;

    let Some(token_row) = token_row else {
        return Err(AuthHandlerError::InvalidCredentials);
    };

    let token_id: uuid::Uuid = token_row
        .try_get("id")
        .map_err(|_| AuthHandlerError::Internal)?;
    let user_id: uuid::Uuid = token_row
        .try_get("user_id")
        .map_err(|_| AuthHandlerError::Internal)?;
    let expires_at: time::OffsetDateTime = token_row
        .try_get("expires_at")
        .map_err(|_| AuthHandlerError::Internal)?;
    let used_at: Option<time::OffsetDateTime> = token_row
        .try_get("used_at")
        .map_err(|_| AuthHandlerError::Internal)?;

    if used_at.is_some() || expires_at <= sqlx::types::time::OffsetDateTime::now_utc() {
        return Err(AuthHandlerError::InvalidCredentials);
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| AuthHandlerError::Internal)?
        .to_string();

    let mut tx = state
        .db
        .begin()
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    sqlx::query(r#"UPDATE "user" SET password_hash = $1 WHERE id = $2"#)
        .bind(&password_hash)
        .bind(user_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    sqlx::query("UPDATE password_reset_token SET used_at = now() WHERE id = $1")
        .bind(token_id)
        .execute(&mut *tx)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    tx.commit().await.map_err(|_| AuthHandlerError::Internal)?;

    Ok(StatusCode::OK)
}

pub async fn list_workspace_users(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<WorkspaceUserResponse>>), AuthHandlerError> {
    let rows = sqlx::query(
        r#"
        SELECT u.id, u.email, u.name, wu.role
        FROM "user" u
        JOIN workspace_user wu ON wu.user_id = u.id
        WHERE wu.workspace_id = $1
        ORDER BY u.email
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AuthHandlerError::Internal)?;

    let users = rows
        .iter()
        .map(|row| -> Result<WorkspaceUserResponse, AuthHandlerError> {
            Ok(WorkspaceUserResponse {
                user_id: UserId(row.try_get("id").map_err(|_| AuthHandlerError::Internal)?),
                email: row
                    .try_get("email")
                    .map_err(|_| AuthHandlerError::Internal)?,
                name: row
                    .try_get("name")
                    .map_err(|_| AuthHandlerError::Internal)?,
                role: row
                    .try_get("role")
                    .map_err(|_| AuthHandlerError::Internal)?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(users)))
}

pub async fn update_workspace_user_role(
    State(state): State<AppState>,
    Path(user_id): Path<UserId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<(StatusCode, Json<WorkspaceUserResponse>), AuthHandlerError> {
    if !auth_user.role.is_at_least(WorkspaceRole::Admin) {
        return Err(AuthHandlerError::InvalidCredentials);
    }

    if user_id == auth_user.user_id && payload.role != WorkspaceRole::Owner {
        return Err(AuthHandlerError::Validation(
            "You cannot demote yourself".to_string(),
        ));
    }

    let row = sqlx::query(
        r#"
        UPDATE workspace_user
        SET role = $3
        WHERE workspace_id = $1 AND user_id = $2
        RETURNING user_id, role
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(user_id.0)
    .bind(payload.role.as_str())
    .fetch_optional(&state.db)
    .await
    .map_err(|_| AuthHandlerError::Internal)?
    .ok_or(AuthHandlerError::NotFound)?;

    let updated_user_id: uuid::Uuid = row
        .try_get("user_id")
        .map_err(|_| AuthHandlerError::Internal)?;

    let user_row = sqlx::query(r#"SELECT email, name FROM "user" WHERE id = $1"#)
        .bind(updated_user_id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    Ok((
        StatusCode::OK,
        Json(WorkspaceUserResponse {
            user_id: UserId(updated_user_id),
            email: user_row
                .try_get("email")
                .map_err(|_| AuthHandlerError::Internal)?,
            name: user_row
                .try_get("name")
                .map_err(|_| AuthHandlerError::Internal)?,
            role: row
                .try_get("role")
                .map_err(|_| AuthHandlerError::Internal)?,
        }),
    ))
}

pub async fn remove_workspace_user(
    State(state): State<AppState>,
    Path(user_id): Path<UserId>,
    auth_user: AuthUser,
) -> Result<StatusCode, AuthHandlerError> {
    if !auth_user.role.is_at_least(WorkspaceRole::Admin) {
        return Err(AuthHandlerError::InvalidCredentials);
    }

    if user_id == auth_user.user_id {
        return Err(AuthHandlerError::Validation(
            "You cannot remove yourself".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM workspace_user WHERE workspace_id = $1 AND user_id = $2")
        .bind(auth_user.workspace_id.0)
        .bind(user_id.0)
        .execute(&state.db)
        .await
        .map_err(|_| AuthHandlerError::Internal)?;

    if result.rows_affected() == 0 {
        return Err(AuthHandlerError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

// This import is required by the `Validate` derive when custom validation is
// used; keep it available for future auth request validations.
#[allow(dead_code)]
type _ValidationError = ValidationError;
