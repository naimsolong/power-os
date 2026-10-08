// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use power_os_domain::{PartyId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PartyType {
    Customer,
    Vendor,
    Other,
}

impl PartyType {
    fn as_str(&self) -> &'static str {
        match self {
            PartyType::Customer => "customer",
            PartyType::Vendor => "vendor",
            PartyType::Other => "other",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePartyRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub party_type: PartyType,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePartyRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub party_type: Option<PartyType>,
}

#[derive(Debug, Deserialize)]
pub struct PartyListQuery {
    pub party_type: Option<PartyType>,
}

#[derive(Debug, Serialize)]
pub struct PartyResponse {
    pub id: PartyId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub party_type: String,
}

fn map_party_row(row: &sqlx::postgres::PgRow) -> Result<PartyResponse, sqlx::Error> {
    Ok(PartyResponse {
        id: PartyId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        name: row.try_get("name")?,
        email: row.try_get("email")?,
        phone: row.try_get("phone")?,
        address: row.try_get("address")?,
        party_type: row.try_get("party_type")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_parties).post(create_party))
        .route("/{id}", get(get_party).patch(update_party).delete(delete_party))
}

pub async fn list_parties(
    State(state): State<AppState>,
    Query(params): Query<PartyListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<PartyResponse>>), ApiError> {
    let party_type = params.party_type.as_ref().map(|pt| pt.as_str().to_string());

    let rows = query(
        r#"
        SELECT id, workspace_id, name, email, phone, address, party_type
        FROM party
        WHERE workspace_id = $1
          AND ($2::text IS NULL OR party_type = $2)
        ORDER BY name
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(party_type)
    .fetch_all(&state.db)
    .await?;

    let parties = rows
        .iter()
        .map(map_party_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(parties)))
}

pub async fn create_party(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreatePartyRequest>,
) -> Result<(StatusCode, Json<PartyResponse>), ApiError> {
    payload.validate()?;

    let party_id = PartyId::new();
    let party_type = payload.party_type.as_str();

    let row = query(
        r#"
        INSERT INTO party (id, workspace_id, name, email, phone, address, party_type)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, workspace_id, name, email, phone, address, party_type
        "#,
    )
    .bind(party_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&payload.name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address)
    .bind(party_type)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_party_row(&row)?)))
}

pub async fn get_party(
    State(state): State<AppState>,
    Path(id): Path<PartyId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<PartyResponse>), ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, name, email, phone, address, party_type
        FROM party
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_party_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_party(
    State(state): State<AppState>,
    Path(id): Path<PartyId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePartyRequest>,
) -> Result<(StatusCode, Json<PartyResponse>), ApiError> {
    payload.validate()?;

    let existing = query("SELECT id FROM party WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    let name = payload.name.as_deref();
    let email = payload.email.as_deref();
    let phone = payload.phone.as_deref();
    let address = payload.address.as_deref();
    let party_type = payload.party_type.as_ref().map(|pt| pt.as_str());

    let row = query(
        r#"
        UPDATE party
        SET
            name = COALESCE($3, name),
            email = COALESCE($4, email),
            phone = COALESCE($5, phone),
            address = COALESCE($6, address),
            party_type = COALESCE($7, party_type),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, name, email, phone, address, party_type
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(name)
    .bind(email)
    .bind(phone)
    .bind(address)
    .bind(party_type)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_party_row(&row)?)))
}

pub async fn delete_party(
    State(state): State<AppState>,
    Path(id): Path<PartyId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM party WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
