// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use power_os_domain::{DealStageId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDealStageRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub position: Option<i32>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDealStageRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct DealStageResponse {
    pub id: DealStageId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub position: i32,
}

fn map_deal_stage_row(row: &sqlx::postgres::PgRow) -> Result<DealStageResponse, sqlx::Error> {
    Ok(DealStageResponse {
        id: DealStageId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        name: row.try_get("name")?,
        position: row.try_get("position")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_deal_stages).post(create_deal_stage))
        .route(
            "/{id}",
            get(get_deal_stage)
                .patch(update_deal_stage)
                .delete(delete_deal_stage),
        )
}

pub async fn list_deal_stages(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<DealStageResponse>>), ApiError> {
    let rows = query(
        r#"
        SELECT id, workspace_id, name, position
        FROM deal_stage
        WHERE workspace_id = $1
        ORDER BY position ASC, name ASC
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_all(&state.db)
    .await?;

    let stages = rows
        .iter()
        .map(map_deal_stage_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(stages)))
}

pub async fn create_deal_stage(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateDealStageRequest>,
) -> Result<(StatusCode, Json<DealStageResponse>), ApiError> {
    payload.validate()?;

    let stage_id = DealStageId::new();
    let position = payload.position.unwrap_or(0);

    let row = query(
        r#"
        INSERT INTO deal_stage (id, workspace_id, name, position)
        VALUES ($1, $2, $3, $4)
        RETURNING id, workspace_id, name, position
        "#,
    )
    .bind(stage_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&payload.name)
    .bind(position)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_deal_stage_row(&row)?)))
}

pub async fn get_deal_stage(
    State(state): State<AppState>,
    Path(id): Path<DealStageId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<DealStageResponse>), ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, name, position
        FROM deal_stage
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_deal_stage_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_deal_stage(
    State(state): State<AppState>,
    Path(id): Path<DealStageId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateDealStageRequest>,
) -> Result<(StatusCode, Json<DealStageResponse>), ApiError> {
    payload.validate()?;

    let existing = query("SELECT id FROM deal_stage WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    let row = query(
        r#"
        UPDATE deal_stage
        SET
            name = COALESCE($3, name),
            position = COALESCE($4, position),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, name, position
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.name.as_deref())
    .bind(payload.position)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_deal_stage_row(&row)?)))
}

pub async fn delete_deal_stage(
    State(state): State<AppState>,
    Path(id): Path<DealStageId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM deal_stage WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
