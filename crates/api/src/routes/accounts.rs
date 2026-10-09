// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use power_os_domain::{AccountId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::OffsetDateTime;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl AccountType {
    fn as_str(&self) -> &'static str {
        match self {
            AccountType::Asset => "asset",
            AccountType::Liability => "liability",
            AccountType::Equity => "equity",
            AccountType::Revenue => "revenue",
            AccountType::Expense => "expense",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: String,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub account_type: AccountType,
    pub parent_account_id: Option<AccountId>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateAccountRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: Option<String>,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,
    pub account_type: Option<AccountType>,
    pub parent_account_id: Option<Option<AccountId>>,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: AccountId,
    pub workspace_id: WorkspaceId,
    pub code: String,
    pub name: String,
    pub account_type: String,
    pub parent_account_id: Option<AccountId>,
    pub parent_code: Option<String>,
    pub parent_name: Option<String>,
    pub is_active: bool,
    pub archived_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

fn map_account_row(row: &sqlx::postgres::PgRow) -> Result<AccountResponse, sqlx::Error> {
    Ok(AccountResponse {
        id: AccountId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        account_type: row.try_get("account_type")?,
        parent_account_id: row.try_get("parent_account_id")?,
        parent_code: row.try_get("parent_code")?,
        parent_name: row.try_get("parent_name")?,
        is_active: row.try_get("is_active")?,
        archived_at: row.try_get("archived_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_accounts).post(create_account))
        .route(
            "/{id}",
            get(get_account).patch(update_account).delete(delete_account),
        )
}

pub async fn list_accounts(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<AccountResponse>>), ApiError> {
    let rows = query(
        r#"
        SELECT
            a.id, a.workspace_id, a.code, a.name, a.account_type,
            a.parent_account_id, a.is_active, a.archived_at,
            a.created_at, a.updated_at,
            p.code as parent_code, p.name as parent_name
        FROM account a
        LEFT JOIN account p ON p.id = a.parent_account_id
        WHERE a.workspace_id = $1
        ORDER BY a.code
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_all(&state.db)
    .await?;

    let accounts = rows
        .iter()
        .map(map_account_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(accounts)))
}

pub async fn create_account(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), ApiError> {
    payload.validate()?;

    let code = payload.code.trim();
    if code.is_empty() {
        return Err(ApiError::BadRequest("Code is required".to_string()));
    }

    ensure_code_unique(&state.db, None, code, auth_user.workspace_id).await?;

    if let Some(parent_id) = payload.parent_account_id {
        ensure_parent_valid(
            &state.db,
            None,
            parent_id,
            auth_user.workspace_id,
        )
        .await?;
    }

    let account_id = AccountId::new();
    let account_type = payload.account_type.as_str();

    let row = query(
        r#"
        INSERT INTO account (
            id, workspace_id, code, name, account_type,
            parent_account_id, is_active, archived_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, true, NULL)
        RETURNING
            id, workspace_id, code, name, account_type,
            parent_account_id, is_active, archived_at,
            created_at, updated_at
        "#,
    )
    .bind(account_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(code)
    .bind(&payload.name)
    .bind(account_type)
    .bind(payload.parent_account_id.map(|p| p.0))
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_account_row_with_parent(&state.db, &row).await?)))
}

pub async fn get_account(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<AccountResponse>), ApiError> {
    let row = query(
        r#"
        SELECT
            a.id, a.workspace_id, a.code, a.name, a.account_type,
            a.parent_account_id, a.is_active, a.archived_at,
            a.created_at, a.updated_at,
            p.code as parent_code, p.name as parent_name
        FROM account a
        LEFT JOIN account p ON p.id = a.parent_account_id
        WHERE a.id = $1 AND a.workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_account_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_account(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateAccountRequest>,
) -> Result<(StatusCode, Json<AccountResponse>), ApiError> {
    payload.validate()?;

    let existing = query(
        "SELECT id FROM account WHERE id = $1 AND workspace_id = $2"
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    if let Some(ref code) = payload.code {
        let trimmed = code.trim();
        if trimmed.is_empty() {
            return Err(ApiError::BadRequest("Code is required".to_string()));
        }
        ensure_code_unique(&state.db, Some(id), trimmed, auth_user.workspace_id).await?;
    }

    if let Some(Some(parent_id)) = payload.parent_account_id {
        ensure_parent_valid(
            &state.db,
            Some(id),
            parent_id,
            auth_user.workspace_id,
        )
        .await?;
    }

    let code = payload.code.as_deref();
    let name = payload.name.as_deref();
    let account_type = payload.account_type.as_ref().map(|t| t.as_str());

    let row = query(
        r#"
        UPDATE account
        SET
            code = COALESCE($3, code),
            name = COALESCE($4, name),
            account_type = COALESCE($5, account_type),
            parent_account_id = COALESCE($6, parent_account_id),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING
            id, workspace_id, code, name, account_type,
            parent_account_id, is_active, archived_at,
            created_at, updated_at
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(code)
    .bind(name)
    .bind(account_type)
    .bind(payload.parent_account_id.as_ref().map(|p| p.map(|pid| pid.0)))
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_account_row_with_parent(&state.db, &row).await?)))
}

pub async fn delete_account(
    State(state): State<AppState>,
    Path(id): Path<AccountId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let referenced: bool = query(
        "SELECT EXISTS (SELECT 1 FROM journal_line WHERE account_id = $1)"
    )
    .bind(id.0)
    .fetch_one(&state.db)
    .await?
    .try_get(0)?;

    if referenced {
        let result = query(
            r#"
            UPDATE account
            SET is_active = false, archived_at = now(), updated_at = now()
            WHERE id = $1 AND workspace_id = $2 AND is_active = true
            "#,
        )
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(ApiError::NotFound);
        }

        return Ok(StatusCode::NO_CONTENT);
    }

    let result = query("DELETE FROM account WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn ensure_code_unique(
    db: &sqlx::PgPool,
    exclude_id: Option<AccountId>,
    code: &str,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists: bool = match exclude_id {
        Some(id) => query(
            "SELECT EXISTS (SELECT 1 FROM account WHERE workspace_id = $1 AND code = $2 AND id <> $3)"
        )
        .bind(workspace_id.0)
        .bind(code)
        .bind(id.0)
        .fetch_one(db)
        .await?
        .try_get(0)?,
        None => query(
            "SELECT EXISTS (SELECT 1 FROM account WHERE workspace_id = $1 AND code = $2)"
        )
        .bind(workspace_id.0)
        .bind(code)
        .fetch_one(db)
        .await?
        .try_get(0)?,
    };

    if exists {
        return Err(ApiError::Conflict(
            "An account with this code already exists in this workspace".to_string(),
        ));
    }

    Ok(())
}

async fn ensure_parent_valid(
    db: &sqlx::PgPool,
    account_id: Option<AccountId>,
    parent_id: AccountId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let parent = query(
        "SELECT id FROM account WHERE id = $1 AND workspace_id = $2"
    )
    .bind(parent_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    if parent.is_none() {
        return Err(ApiError::BadRequest(
            "Selected parent account does not exist in this workspace".to_string(),
        ));
    }

    if let Some(id) = account_id {
        if id.0 == parent_id.0 {
            return Err(ApiError::BadRequest(
                "An account cannot be its own parent".to_string(),
            ));
        }

        let is_descendant: bool = query(
            r#"
            WITH RECURSIVE descendants AS (
                SELECT id, parent_account_id
                FROM account
                WHERE id = $1
                UNION ALL
                SELECT a.id, a.parent_account_id
                FROM account a
                INNER JOIN descendants d ON a.parent_account_id = d.id
            )
            SELECT EXISTS (SELECT 1 FROM descendants WHERE id = $2)
            "#,
        )
        .bind(id.0)
        .bind(parent_id.0)
        .fetch_one(db)
        .await?
        .try_get(0)?;

        if is_descendant {
            return Err(ApiError::BadRequest(
                "Parent cannot be a descendant of this account".to_string(),
            ));
        }
    }

    Ok(())
}

async fn map_account_row_with_parent(
    db: &sqlx::PgPool,
    row: &sqlx::postgres::PgRow,
) -> Result<AccountResponse, ApiError> {
    let parent_account_id: Option<AccountId> = row.try_get("parent_account_id")?;

    let (parent_code, parent_name) = if let Some(pid) = parent_account_id {
        let parent = query("SELECT code, name FROM account WHERE id = $1")
            .bind(pid.0)
            .fetch_optional(db)
            .await?;

        match parent {
            Some(parent) => (
                parent.try_get("code").ok(),
                parent.try_get("name").ok(),
            ),
            None => (None, None),
        }
    } else {
        (None, None)
    };

    Ok(AccountResponse {
        id: AccountId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        account_type: row.try_get("account_type")?,
        parent_account_id,
        parent_code,
        parent_name,
        is_active: row.try_get("is_active")?,
        archived_at: row.try_get("archived_at")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
