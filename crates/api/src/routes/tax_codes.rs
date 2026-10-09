// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, Zero};
use power_os_domain::{TaxCodeId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxType {
    Sst,
    ServiceTax,
}

impl TaxType {
    fn as_str(&self) -> &'static str {
        match self {
            TaxType::Sst => "sst",
            TaxType::ServiceTax => "service_tax",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTaxCodeRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: String,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    #[validate(custom(function = "validate_rate"))]
    pub rate: BigDecimal,
    pub tax_type: TaxType,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateTaxCodeRequest {
    #[validate(length(min = 1, message = "Code is required"))]
    pub code: Option<String>,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: Option<String>,
    #[validate(custom(function = "validate_rate"))]
    pub rate: Option<BigDecimal>,
    pub tax_type: Option<TaxType>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct TaxCodeResponse {
    pub id: TaxCodeId,
    pub workspace_id: WorkspaceId,
    pub code: String,
    pub description: String,
    pub rate: BigDecimal,
    pub tax_type: String,
    pub is_active: bool,
}

fn validate_rate(rate: &BigDecimal) -> Result<(), validator::ValidationError> {
    if rate < &BigDecimal::zero() {
        let mut err = validator::ValidationError::new("rate");
        err.message = Some("Rate must be 0 or greater".into());
        return Err(err);
    }
    Ok(())
}

fn map_tax_code_row(row: &sqlx::postgres::PgRow) -> Result<TaxCodeResponse, sqlx::Error> {
    Ok(TaxCodeResponse {
        id: TaxCodeId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        code: row.try_get("code")?,
        description: row.try_get("description")?,
        rate: row.try_get("rate")?,
        tax_type: row.try_get("tax_type")?,
        is_active: row.try_get("is_active")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tax_codes).post(create_tax_code))
        .route(
            "/{id}",
            get(get_tax_code)
                .patch(update_tax_code)
                .delete(delete_tax_code),
        )
}

pub async fn list_tax_codes(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<TaxCodeResponse>>), ApiError> {
    let rows = query(
        r#"
        SELECT id, workspace_id, code, description, rate, tax_type, is_active
        FROM tax_code
        WHERE workspace_id = $1 AND is_active = true
        ORDER BY code
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_all(&state.db)
    .await?;

    let tax_codes = rows
        .iter()
        .map(map_tax_code_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(tax_codes)))
}

pub async fn create_tax_code(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateTaxCodeRequest>,
) -> Result<(StatusCode, Json<TaxCodeResponse>), ApiError> {
    payload.validate()?;

    let tax_code_id = TaxCodeId::new();
    let tax_type = payload.tax_type.as_str();

    let row = query(
        r#"
        INSERT INTO tax_code (id, workspace_id, code, description, rate, tax_type)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, workspace_id, code, description, rate, tax_type, is_active
        "#,
    )
    .bind(tax_code_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&payload.code)
    .bind(&payload.description)
    .bind(&payload.rate)
    .bind(tax_type)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ApiError::Conflict("Tax code already exists for this workspace".to_string())
        }
        _ => ApiError::from(e),
    })?;

    Ok((StatusCode::CREATED, Json(map_tax_code_row(&row)?)))
}

pub async fn get_tax_code(
    State(state): State<AppState>,
    Path(id): Path<TaxCodeId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<TaxCodeResponse>), ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, code, description, rate, tax_type, is_active
        FROM tax_code
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_tax_code_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_tax_code(
    State(state): State<AppState>,
    Path(id): Path<TaxCodeId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateTaxCodeRequest>,
) -> Result<(StatusCode, Json<TaxCodeResponse>), ApiError> {
    payload.validate()?;

    let existing = query("SELECT id FROM tax_code WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    let code = payload.code.as_deref();
    let description = payload.description.as_deref();
    let rate = payload.rate.as_ref();
    let tax_type = payload.tax_type.as_ref().map(|t| t.as_str());
    let is_active = payload.is_active;

    let row = query(
        r#"
        UPDATE tax_code
        SET
            code = COALESCE($3, code),
            description = COALESCE($4, description),
            rate = COALESCE($5, rate),
            tax_type = COALESCE($6, tax_type),
            is_active = COALESCE($7, is_active),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, code, description, rate, tax_type, is_active
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(code)
    .bind(description)
    .bind(rate)
    .bind(tax_type)
    .bind(is_active)
    .fetch_one(&state.db)
    .await
    .map_err(|e| match &e {
        sqlx::Error::Database(db_err) if db_err.is_unique_violation() => {
            ApiError::Conflict("Tax code already exists for this workspace".to_string())
        }
        _ => ApiError::from(e),
    })?;

    Ok((StatusCode::OK, Json(map_tax_code_row(&row)?)))
}

pub async fn delete_tax_code(
    State(state): State<AppState>,
    Path(id): Path<TaxCodeId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query(
        "UPDATE tax_code SET is_active = false, updated_at = now() WHERE id = $1 AND workspace_id = $2 AND is_active = true"
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&state.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
