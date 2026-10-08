// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, patch, post},
    Json, Router,
};
use power_os_domain::{DealId, DealStageId, PartyId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDealRequest {
    pub party_id: PartyId,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub stage_id: Option<DealStageId>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub expected_close_date: Option<Date>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDealRequest {
    pub party_id: Option<PartyId>,
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,
    pub stage_id: Option<DealStageId>,
    pub value: Option<f64>,
    pub currency: Option<String>,
    pub expected_close_date: Option<Date>,
}

#[derive(Debug, Deserialize)]
pub struct DealListQuery {
    pub party_id: Option<PartyId>,
    pub stage_id: Option<DealStageId>,
}

#[derive(Debug, Serialize)]
pub struct DealResponse {
    pub id: DealId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub name: String,
    pub stage_id: Option<DealStageId>,
    pub stage_name: Option<String>,
    pub value: Option<f64>,
    pub currency: String,
    pub expected_close_date: Option<Date>,
}

fn map_deal_row(row: &sqlx::postgres::PgRow) -> Result<DealResponse, sqlx::Error> {
    Ok(DealResponse {
        id: DealId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: PartyId(row.try_get("party_id")?),
        name: row.try_get("name")?,
        stage_id: row.try_get("stage_id")?,
        stage_name: row.try_get("stage_name")?,
        value: row.try_get("value")?,
        currency: row.try_get("currency")?,
        expected_close_date: row.try_get("expected_close_date")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_deals).post(create_deal))
        .route("/:id", get(get_deal).patch(update_deal).delete(delete_deal))
}

pub async fn list_deals(
    State(state): State<AppState>,
    Query(params): Query<DealListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<DealResponse>>), ApiError> {
    let rows = query(
        r#"
        SELECT
            d.id,
            d.workspace_id,
            d.party_id,
            d.name,
            d.stage_id,
            ds.name AS stage_name,
            d.value,
            d.currency,
            d.expected_close_date
        FROM deal d
        LEFT JOIN deal_stage ds ON ds.id = d.stage_id AND ds.workspace_id = d.workspace_id
        WHERE d.workspace_id = $1
          AND ($2::uuid IS NULL OR d.party_id = $2)
          AND ($3::uuid IS NULL OR d.stage_id = $3)
        ORDER BY d.created_at DESC
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.party_id.map(|id| id.0))
    .bind(params.stage_id.map(|id| id.0))
    .fetch_all(&state.db)
    .await?;

    let deals = rows
        .iter()
        .map(map_deal_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(deals)))
}

async fn ensure_party_in_workspace(
    db: &sqlx::PgPool,
    party_id: PartyId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists = query("SELECT 1 FROM party WHERE id = $1 AND workspace_id = $2")
        .bind(party_id.0)
        .bind(workspace_id.0)
        .fetch_optional(db)
        .await?;

    if exists.is_none() {
        return Err(ApiError::BadRequest("Party not found in workspace".to_string()));
    }
    Ok(())
}

async fn ensure_stage_in_workspace(
    db: &sqlx::PgPool,
    stage_id: DealStageId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists = query("SELECT 1 FROM deal_stage WHERE id = $1 AND workspace_id = $2")
        .bind(stage_id.0)
        .bind(workspace_id.0)
        .fetch_optional(db)
        .await?;

    if exists.is_none() {
        return Err(ApiError::BadRequest(
            "Deal stage not found in workspace".to_string(),
        ));
    }
    Ok(())
}

pub async fn create_deal(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateDealRequest>,
) -> Result<(StatusCode, Json<DealResponse>), ApiError> {
    payload.validate()?;

    ensure_party_in_workspace(&state.db, payload.party_id, auth_user.workspace_id).await?;

    if let Some(stage_id) = payload.stage_id {
        ensure_stage_in_workspace(&state.db, stage_id, auth_user.workspace_id).await?;
    }

    let deal_id = DealId::new();
    let currency = payload.currency.as_deref().unwrap_or("MYR");

    let row = query(
        r#"
        INSERT INTO deal (id, workspace_id, party_id, name, stage_id, value, currency, expected_close_date)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING
            id,
            workspace_id,
            party_id,
            name,
            stage_id,
            (SELECT name FROM deal_stage WHERE id = stage_id AND workspace_id = deal.workspace_id) AS stage_name,
            value,
            currency,
            expected_close_date
        "#,
    )
    .bind(deal_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.0)
    .bind(&payload.name)
    .bind(payload.stage_id.map(|id| id.0))
    .bind(payload.value)
    .bind(currency)
    .bind(payload.expected_close_date)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_deal_row(&row)?)))
}

pub async fn get_deal(
    State(state): State<AppState>,
    Path(id): Path<DealId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<DealResponse>), ApiError> {
    let row = query(
        r#"
        SELECT
            d.id,
            d.workspace_id,
            d.party_id,
            d.name,
            d.stage_id,
            ds.name AS stage_name,
            d.value,
            d.currency,
            d.expected_close_date
        FROM deal d
        LEFT JOIN deal_stage ds ON ds.id = d.stage_id AND ds.workspace_id = d.workspace_id
        WHERE d.id = $1 AND d.workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_deal_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_deal(
    State(state): State<AppState>,
    Path(id): Path<DealId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateDealRequest>,
) -> Result<(StatusCode, Json<DealResponse>), ApiError> {
    payload.validate()?;

    let existing = query("SELECT id FROM deal WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    if let Some(stage_id) = payload.stage_id {
        ensure_stage_in_workspace(&state.db, stage_id, auth_user.workspace_id).await?;
    }

    let row = query(
        r#"
        UPDATE deal
        SET
            party_id = COALESCE($3, party_id),
            name = COALESCE($4, name),
            stage_id = COALESCE($5, stage_id),
            value = COALESCE($6, value),
            currency = COALESCE($7, currency),
            expected_close_date = COALESCE($8, expected_close_date),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING
            id,
            workspace_id,
            party_id,
            name,
            stage_id,
            (SELECT name FROM deal_stage WHERE id = deal.stage_id AND workspace_id = deal.workspace_id) AS stage_name,
            value,
            currency,
            expected_close_date
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|id| id.0))
    .bind(payload.name.as_deref())
    .bind(payload.stage_id.map(|id| id.0))
    .bind(payload.value)
    .bind(payload.currency.as_deref())
    .bind(payload.expected_close_date)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_deal_row(&row)?)))
}

pub async fn delete_deal(
    State(state): State<AppState>,
    Path(id): Path<DealId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM deal WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
