// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use power_os_domain::{AccountingPeriodId, FiscalYearId, UserId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::{Date, Duration, Month};
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateFiscalYearRequest {
    #[validate(length(min = 1, message = "Name is required"))]
    pub name: String,
    pub start_date: Date,
    pub end_date: Date,
}

#[derive(Clone, Debug, Serialize)]
pub struct AccountingPeriodResponse {
    pub id: AccountingPeriodId,
    pub workspace_id: WorkspaceId,
    pub fiscal_year_id: FiscalYearId,
    pub name: String,
    pub start_date: Date,
    pub end_date: Date,
    pub is_closed: bool,
    pub closed_at: Option<time::OffsetDateTime>,
    pub closed_by_user_id: Option<UserId>,
}

#[derive(Debug, Serialize)]
pub struct FiscalYearResponse {
    pub id: FiscalYearId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub start_date: Date,
    pub end_date: Date,
    pub is_closed: bool,
    pub periods: Vec<AccountingPeriodResponse>,
}

fn map_fiscal_year_row(row: &sqlx::postgres::PgRow) -> Result<FiscalYearResponse, sqlx::Error> {
    Ok(FiscalYearResponse {
        id: FiscalYearId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        name: row.try_get("name")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        is_closed: row.try_get("is_closed")?,
        periods: Vec::new(),
    })
}

fn map_period_row(row: &sqlx::postgres::PgRow) -> Result<AccountingPeriodResponse, sqlx::Error> {
    Ok(AccountingPeriodResponse {
        id: AccountingPeriodId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        fiscal_year_id: FiscalYearId(row.try_get("fiscal_year_id")?),
        name: row.try_get("name")?,
        start_date: row.try_get("start_date")?,
        end_date: row.try_get("end_date")?,
        is_closed: row.try_get("is_closed")?,
        closed_at: row.try_get("closed_at")?,
        closed_by_user_id: row.try_get("closed_by_user_id")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_fiscal_years).post(create_fiscal_year))
}

pub fn accounting_periods_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_accounting_periods))
        .route("/{id}/close", post(close_period))
        .route("/{id}/reopen", post(reopen_period))
}

fn month_from_u8(month: u8) -> Result<Month, ApiError> {
    Month::try_from(month).map_err(|_| ApiError::Internal)
}

fn add_months(year: i32, month: u8, months: i32) -> Result<(i32, u8), ApiError> {
    let month_idx = (year * 12 + (month as i32 - 1)) + months;
    let new_year = month_idx / 12;
    let new_month = (month_idx % 12) + 1;
    if !(1..=12).contains(&new_month) {
        return Err(ApiError::Internal);
    }
    Ok((new_year, new_month as u8))
}

fn month_start(year: i32, month: u8) -> Result<Date, ApiError> {
    let month = month_from_u8(month)?;
    Date::from_calendar_date(year, month, 1).map_err(|_| ApiError::Internal)
}

fn next_month_start(year: i32, month: u8) -> Result<Date, ApiError> {
    let (next_year, next_month) = add_months(year, month, 1)?;
    month_start(next_year, next_month)
}

fn end_of_month(year: i32, month: u8) -> Result<Date, ApiError> {
    let next = next_month_start(year, month)?;
    Ok(next - Duration::days(1))
}

fn period_name(year: i32, month: u8) -> Result<String, ApiError> {
    let month_enum = month_from_u8(month)?;
    Ok(format!("{} {}", month_enum, year))
}

async fn fetch_periods_for_workspace(
    db: &sqlx::PgPool,
    workspace_id: WorkspaceId,
) -> Result<Vec<AccountingPeriodResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, workspace_id, fiscal_year_id, name, start_date, end_date,
               is_closed, closed_at, closed_by_user_id
        FROM accounting_period
        WHERE workspace_id = $1
        ORDER BY start_date ASC
        "#,
    )
    .bind(workspace_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_period_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

pub async fn list_fiscal_years(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<FiscalYearResponse>>), ApiError> {
    let rows = query(
        r#"
        SELECT id, workspace_id, name, start_date, end_date, is_closed
        FROM fiscal_year
        WHERE workspace_id = $1
        ORDER BY start_date DESC
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .fetch_all(&state.db)
    .await?;

    let mut fiscal_years: Vec<FiscalYearResponse> = rows
        .iter()
        .map(map_fiscal_year_row)
        .collect::<Result<Vec<_>, _>>()?;

    let periods = fetch_periods_for_workspace(&state.db, auth_user.workspace_id).await?;

    for year in &mut fiscal_years {
        year.periods = periods
            .iter()
            .filter(|p| p.fiscal_year_id == year.id)
            .cloned()
            .collect();
    }

    Ok((StatusCode::OK, Json(fiscal_years)))
}

pub async fn list_accounting_periods(
    State(state): State<AppState>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<AccountingPeriodResponse>>), ApiError> {
    let periods = fetch_periods_for_workspace(&state.db, auth_user.workspace_id).await?;
    Ok((StatusCode::OK, Json(periods)))
}

async fn ensure_no_fiscal_year_overlap(
    db: &sqlx::PgPool,
    workspace_id: WorkspaceId,
    start_date: Date,
    end_date: Date,
    exclude_id: Option<FiscalYearId>,
) -> Result<(), ApiError> {
    let existing = query(
        r#"
        SELECT id
        FROM fiscal_year
        WHERE workspace_id = $1
          AND ($4::uuid IS NULL OR id != $4)
          AND start_date <= $3
          AND end_date >= $2
        "#,
    )
    .bind(workspace_id.0)
    .bind(start_date)
    .bind(end_date)
    .bind(exclude_id.map(|id| id.0))
    .fetch_optional(db)
    .await?;

    if existing.is_some() {
        return Err(ApiError::Conflict(
            "Date range overlaps an existing fiscal year".to_string(),
        ));
    }

    Ok(())
}

async fn generate_monthly_periods(
    db: &mut sqlx::PgConnection,
    workspace_id: WorkspaceId,
    fiscal_year_id: FiscalYearId,
    start_date: Date,
) -> Result<(), ApiError> {
    let start_year = start_date.year();
    let start_month = start_date.month() as u8;

    for i in 0..12 {
        let (year, month) = add_months(start_year, start_month, i)?;
        let period_start = month_start(year, month)?;
        let period_end = end_of_month(year, month)?;
        let name = period_name(year, month)?;

        query(
            r#"
            INSERT INTO accounting_period
                (id, workspace_id, fiscal_year_id, name, start_date, end_date)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(AccountingPeriodId::new().0)
        .bind(workspace_id.0)
        .bind(fiscal_year_id.0)
        .bind(&name)
        .bind(period_start)
        .bind(period_end)
        .execute(&mut *db)
        .await?;
    }

    Ok(())
}

pub async fn create_fiscal_year(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateFiscalYearRequest>,
) -> Result<(StatusCode, Json<FiscalYearResponse>), ApiError> {
    payload.validate()?;

    if payload.end_date < payload.start_date {
        return Err(ApiError::BadRequest(
            "End date must be on or after start date".to_string(),
        ));
    }

    ensure_no_fiscal_year_overlap(
        &state.db,
        auth_user.workspace_id,
        payload.start_date,
        payload.end_date,
        None,
    )
    .await?;

    let fiscal_year_id = FiscalYearId::new();

    let mut tx = state.db.begin().await?;

    let row = query(
        r#"
        INSERT INTO fiscal_year (id, workspace_id, name, start_date, end_date)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, workspace_id, name, start_date, end_date, is_closed
        "#,
    )
    .bind(fiscal_year_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&payload.name)
    .bind(payload.start_date)
    .bind(payload.end_date)
    .fetch_one(&mut *tx)
    .await?;

    generate_monthly_periods(
        &mut *tx,
        auth_user.workspace_id,
        fiscal_year_id,
        payload.start_date,
    )
    .await?;

    tx.commit().await?;

    let mut fiscal_year = map_fiscal_year_row(&row)?;
    fiscal_year.periods = fetch_periods_for_workspace(&state.db, auth_user.workspace_id)
        .await?
        .into_iter()
        .filter(|p| p.fiscal_year_id == fiscal_year.id)
        .collect();

    Ok((StatusCode::CREATED, Json(fiscal_year)))
}

pub async fn close_period(
    State(state): State<AppState>,
    Path(id): Path<AccountingPeriodId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<AccountingPeriodResponse>), ApiError> {
    let row = query(
        r#"
        UPDATE accounting_period
        SET is_closed = true,
            closed_at = now(),
            closed_by_user_id = $3,
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, fiscal_year_id, name, start_date, end_date,
                  is_closed, closed_at, closed_by_user_id
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(auth_user.user_id.0)
    .fetch_optional(&state.db)
    .await?;

    match row {
        Some(row) => Ok((StatusCode::OK, Json(map_period_row(&row)?))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn reopen_period(
    State(state): State<AppState>,
    Path(id): Path<AccountingPeriodId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<AccountingPeriodResponse>), ApiError> {
    let period = query(
        r#"
        SELECT start_date
        FROM accounting_period
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_optional(&state.db)
    .await?;

    let start_date: Date = match period {
        Some(row) => row.try_get("start_date")?,
        None => return Err(ApiError::NotFound),
    };

    let later_closed = query(
        r#"
        SELECT 1
        FROM accounting_period
        WHERE workspace_id = $1
          AND start_date > $2
          AND is_closed = true
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(start_date)
    .fetch_optional(&state.db)
    .await?;

    if later_closed.is_some() {
        return Err(ApiError::Conflict(
            "Cannot reopen a period when a later period is closed".to_string(),
        ));
    }

    let row = query(
        r#"
        UPDATE accounting_period
        SET is_closed = false,
            closed_at = NULL,
            closed_by_user_id = NULL,
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        RETURNING id, workspace_id, fiscal_year_id, name, start_date, end_date,
                  is_closed, closed_at, closed_by_user_id
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_period_row(&row)?)))
}

/// Seeds a fiscal year beginning on the first day of the current calendar year
/// with 12 monthly accounting periods. Intended to be called once during
/// workspace creation.
pub async fn seed_fiscal_year(
    pool: &sqlx::PgPool,
    workspace_id: WorkspaceId,
    user_id: UserId,
) -> Result<(), ApiError> {
    let now = time::OffsetDateTime::now_utc();
    let year = now.year();
    let start_date = Date::from_calendar_date(year, Month::January, 1)
        .map_err(|_| ApiError::Internal)?;
    let end_date = end_of_month(year, 12)?;

    let existing = query("SELECT id FROM fiscal_year WHERE workspace_id = $1")
        .bind(workspace_id.0)
        .fetch_optional(pool)
        .await?;

    if existing.is_some() {
        return Ok(());
    }

    let fiscal_year_id = FiscalYearId::new();

    let mut tx = pool.begin().await?;

    query(
        r#"
        INSERT INTO fiscal_year (id, workspace_id, name, start_date, end_date)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(fiscal_year_id.0)
    .bind(workspace_id.0)
    .bind(format!("Fiscal Year {}", year))
    .bind(start_date)
    .bind(end_date)
    .execute(&mut *tx)
    .await?;

    generate_monthly_periods(&mut *tx, workspace_id, fiscal_year_id, start_date).await?;

    tx.commit().await?;

    // user_id is accepted for API symmetry but the seeded periods are left open;
    // the caller may use it for audit logging later.
    let _ = user_id;

    Ok(())
}
