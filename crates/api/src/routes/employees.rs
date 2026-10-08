// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use power_os_domain::{EmployeeId, PartyId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EmployeeStatus {
    Active,
    Inactive,
}

impl EmployeeStatus {
    fn as_str(&self) -> &'static str {
        match self {
            EmployeeStatus::Active => "active",
            EmployeeStatus::Inactive => "inactive",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateEmployeeRequest {
    #[validate(length(min = 1, message = "Employee code is required"))]
    pub employee_code: String,
    pub party_id: PartyId,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub hire_date: Option<Date>,
    pub status: Option<EmployeeStatus>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateEmployeeRequest {
    #[validate(length(min = 1, message = "Employee code is required"))]
    pub employee_code: Option<String>,
    pub party_id: Option<PartyId>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub hire_date: Option<Date>,
    pub status: Option<EmployeeStatus>,
}

#[derive(Debug, Deserialize)]
pub struct EmployeeListQuery {
    pub status: Option<EmployeeStatus>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmployeeResponse {
    pub id: EmployeeId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub employee_code: String,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub hire_date: Option<Date>,
    pub status: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

fn map_employee_row(row: &sqlx::postgres::PgRow) -> Result<EmployeeResponse, sqlx::Error> {
    Ok(EmployeeResponse {
        id: EmployeeId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: PartyId(row.try_get("party_id")?),
        employee_code: row.try_get("employee_code")?,
        job_title: row.try_get("job_title")?,
        department: row.try_get("department")?,
        hire_date: row.try_get("hire_date")?,
        status: row.try_get("status")?,
        name: row.try_get("name")?,
        email: row.try_get("email")?,
        phone: row.try_get("phone")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_employees).post(create_employee))
        .route(
            "/{id}",
            get(get_employee)
                .patch(update_employee)
                .delete(delete_employee),
        )
}

pub async fn list_employees(
    State(state): State<AppState>,
    Query(params): Query<EmployeeListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<EmployeeResponse>>), ApiError> {
    let status = params.status.as_ref().map(|s| s.as_str().to_string());

    let rows = query(
        r#"
        SELECT
            e.id, e.workspace_id, e.party_id, e.employee_code, e.job_title,
            e.department, e.hire_date, e.status,
            p.name, p.email, p.phone
        FROM employee e
        JOIN party p ON p.id = e.party_id
        WHERE e.workspace_id = $1
          AND ($2::text IS NULL OR e.status = $2)
          AND (
              $3::text IS NULL
              OR e.employee_code ILIKE '%' || $3 || '%'
              OR e.job_title ILIKE '%' || $3 || '%'
              OR e.department ILIKE '%' || $3 || '%'
              OR p.name ILIKE '%' || $3 || '%'
          )
        ORDER BY p.name
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(status)
    .bind(params.search)
    .fetch_all(&state.db)
    .await?;

    let employees = rows
        .iter()
        .map(map_employee_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(employees)))
}

pub async fn create_employee(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateEmployeeRequest>,
) -> Result<(StatusCode, Json<EmployeeResponse>), ApiError> {
    payload.validate()?;

    ensure_party_in_workspace(&state.db, payload.party_id, auth_user.workspace_id).await?;

    let status = payload
        .status
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("active");

    let row = query(
        r#"
        INSERT INTO employee (
            workspace_id, party_id, employee_code, job_title,
            department, hire_date, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id, workspace_id, party_id, employee_code, job_title,
            department, hire_date, status
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.0)
    .bind(&payload.employee_code)
    .bind(&payload.job_title)
    .bind(&payload.department)
    .bind(payload.hire_date)
    .bind(status)
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(map_employee_row_with_party(&state.db, &row).await?),
    ))
}

pub async fn get_employee(
    State(state): State<AppState>,
    Path(id): Path<EmployeeId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<EmployeeResponse>), ApiError> {
    let row = query(
        r#"
        SELECT
            e.id, e.workspace_id, e.party_id, e.employee_code, e.job_title,
            e.department, e.hire_date, e.status,
            p.name, p.email, p.phone
        FROM employee e
        JOIN party p ON p.id = e.party_id
        WHERE e.id = $1 AND e.workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_employee_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn update_employee(
    State(state): State<AppState>,
    Path(id): Path<EmployeeId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateEmployeeRequest>,
) -> Result<(StatusCode, Json<EmployeeResponse>), ApiError> {
    payload.validate()?;

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    let existing = query("SELECT id FROM employee WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .fetch_optional(&state.db)
        .await?;

    if existing.is_none() {
        return Err(ApiError::NotFound);
    }

    let row = query(
        r#"
        UPDATE employee
        SET
            party_id = COALESCE($3, party_id),
            employee_code = COALESCE($4, employee_code),
            job_title = COALESCE($5, job_title),
            department = COALESCE($6, department),
            hire_date = COALESCE($7, hire_date),
            status = COALESCE($8, status),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING
            id, workspace_id, party_id, employee_code, job_title,
            department, hire_date, status
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|p| p.0))
    .bind(&payload.employee_code)
    .bind(&payload.job_title)
    .bind(&payload.department)
    .bind(payload.hire_date)
    .bind(payload.status.as_ref().map(|s| s.as_str()))
    .fetch_one(&state.db)
    .await?;

    Ok((
        StatusCode::OK,
        Json(map_employee_row_with_party(&state.db, &row).await?),
    ))
}

pub async fn delete_employee(
    State(state): State<AppState>,
    Path(id): Path<EmployeeId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM employee WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
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
        return Err(ApiError::BadRequest(
            "Selected party does not exist in this workspace".to_string(),
        ));
    }

    Ok(())
}

async fn map_employee_row_with_party(
    db: &sqlx::PgPool,
    row: &sqlx::postgres::PgRow,
) -> Result<EmployeeResponse, ApiError> {
    let party_id: PartyId = PartyId(row.try_get("party_id")?);
    let party = query("SELECT name, email, phone FROM party WHERE id = $1")
        .bind(party_id.0)
        .fetch_one(db)
        .await?;

    Ok(EmployeeResponse {
        id: EmployeeId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id,
        employee_code: row.try_get("employee_code")?,
        job_title: row.try_get("job_title")?,
        department: row.try_get("department")?,
        hire_date: row.try_get("hire_date")?,
        status: row.try_get("status")?,
        name: party.try_get("name")?,
        email: party.try_get("email")?,
        phone: party.try_get("phone")?,
    })
}
