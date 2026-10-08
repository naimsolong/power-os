// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use bigdecimal::{BigDecimal, Zero};
use power_os_domain::{AccountId, JournalEntryId, JournalLineId, PartyId, WorkspaceId};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalEntryStatus {
    Draft,
    Posted,
    Cancelled,
}

impl JournalEntryStatus {
    fn as_str(&self) -> &'static str {
        match self {
            JournalEntryStatus::Draft => "draft",
            JournalEntryStatus::Posted => "posted",
            JournalEntryStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateJournalLineRequest {
    pub account_id: AccountId,
    pub party_id: Option<PartyId>,
    pub description: Option<String>,
    pub debit: Option<BigDecimal>,
    pub credit: Option<BigDecimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateJournalEntryRequest {
    pub entry_date: Date,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub status: Option<JournalEntryStatus>,
    pub lines: Vec<CreateJournalLineRequest>,
}

#[derive(Debug, Deserialize)]
pub struct JournalEntryListQuery {
    pub status: Option<JournalEntryStatus>,
}

#[derive(Debug, Serialize)]
pub struct JournalLineResponse {
    pub id: JournalLineId,
    pub journal_entry_id: JournalEntryId,
    pub account_id: AccountId,
    pub party_id: Option<PartyId>,
    pub description: Option<String>,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct JournalEntryResponse {
    pub id: JournalEntryId,
    pub workspace_id: WorkspaceId,
    pub entry_date: Date,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub status: String,
    pub lines: Vec<JournalLineResponse>,
}

fn map_journal_entry_row(row: &sqlx::postgres::PgRow) -> Result<JournalEntryResponse, sqlx::Error> {
    Ok(JournalEntryResponse {
        id: JournalEntryId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        entry_date: row.try_get("entry_date")?,
        reference: row.try_get("reference")?,
        description: row.try_get("description")?,
        status: row.try_get("status")?,
        lines: Vec::new(),
    })
}

fn map_journal_line_row(row: &sqlx::postgres::PgRow) -> Result<JournalLineResponse, sqlx::Error> {
    Ok(JournalLineResponse {
        id: JournalLineId(row.try_get("id")?),
        journal_entry_id: JournalEntryId(row.try_get("journal_entry_id")?),
        account_id: AccountId(row.try_get("account_id")?),
        party_id: row.try_get("party_id")?,
        description: row.try_get("description")?,
        debit: row.try_get("debit")?,
        credit: row.try_get("credit")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_journal_entries).post(create_journal_entry))
        .route("/{id}", get(get_journal_entry).delete(delete_journal_entry))
}

async fn ensure_account_in_workspace(
    db: &sqlx::PgPool,
    account_id: AccountId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists = query("SELECT 1 FROM account WHERE id = $1 AND workspace_id = $2")
        .bind(account_id.0)
        .bind(workspace_id.0)
        .fetch_optional(db)
        .await?;

    if exists.is_none() {
        return Err(ApiError::BadRequest(
            "Account not found in workspace".to_string(),
        ));
    }
    Ok(())
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
            "Party not found in workspace".to_string(),
        ));
    }
    Ok(())
}

async fn fetch_journal_entry(
    db: &sqlx::PgPool,
    journal_entry_id: JournalEntryId,
    workspace_id: WorkspaceId,
) -> Result<Option<JournalEntryResponse>, ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, entry_date, reference, description, status
        FROM journal_entry
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => {
            let mut entry = map_journal_entry_row(&row)?;
            let lines = fetch_journal_lines(db, journal_entry_id).await?;
            entry.lines = lines;
            Ok(Some(entry))
        }
        None => Ok(None),
    }
}

async fn fetch_journal_lines(
    db: &sqlx::PgPool,
    journal_entry_id: JournalEntryId,
) -> Result<Vec<JournalLineResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, journal_entry_id, account_id, party_id, description, debit, credit
        FROM journal_line
        WHERE journal_entry_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(journal_entry_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_journal_line_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

pub async fn list_journal_entries(
    State(state): State<AppState>,
    Query(params): Query<JournalEntryListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<JournalEntryResponse>>), ApiError> {
    let status = params.status.as_ref().map(|s| s.as_str().to_string());

    let rows = query(
        r#"
        SELECT id, workspace_id, entry_date, reference, description, status
        FROM journal_entry
        WHERE workspace_id = $1
          AND ($2::text IS NULL OR status = $2)
        ORDER BY entry_date DESC, reference
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(status)
    .fetch_all(&state.db)
    .await?;

    let mut entries = Vec::with_capacity(rows.len());
    for row in &rows {
        let mut entry = map_journal_entry_row(row)?;
        entry.lines = fetch_journal_lines(&state.db, entry.id).await?;
        entries.push(entry);
    }

    Ok((StatusCode::OK, Json(entries)))
}

pub async fn create_journal_entry(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateJournalEntryRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), ApiError> {
    payload.validate()?;

    if payload.lines.len() < 2 {
        return Err(ApiError::BadRequest(
            "At least two journal lines are required".to_string(),
        ));
    }

    let zero = BigDecimal::zero();

    let total_debit: BigDecimal = payload
        .lines
        .iter()
        .map(|line| line.debit.as_ref().unwrap_or(&zero))
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    let total_credit: BigDecimal = payload
        .lines
        .iter()
        .map(|line| line.credit.as_ref().unwrap_or(&zero))
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    if total_debit != total_credit {
        return Err(ApiError::BadRequest(
            "Journal entry debits must equal credits".to_string(),
        ));
    }

    for line in &payload.lines {
        ensure_account_in_workspace(&state.db, line.account_id, auth_user.workspace_id).await?;
        if let Some(party_id) = line.party_id {
            ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
        }
    }

    let status = payload
        .status
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("draft");
    let journal_entry_id = JournalEntryId::new();

    let mut tx = state.db.begin().await?;

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.entry_date)
    .bind(payload.reference.as_deref())
    .bind(payload.description.as_deref())
    .bind(status)
    .execute(&mut *tx)
    .await?;

    for line in &payload.lines {
        let debit = line.debit.as_ref().unwrap_or(&zero);
        let credit = line.credit.as_ref().unwrap_or(&zero);

        query(
            r#"
            INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(journal_entry_id.0)
        .bind(line.account_id.0)
        .bind(line.party_id.map(|p| p.0))
        .bind(line.description.as_deref())
        .bind(debit)
        .bind(credit)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let entry = fetch_journal_entry(&state.db, journal_entry_id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(entry)))
}

pub async fn get_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<JournalEntryId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<JournalEntryResponse>), ApiError> {
    let entry = fetch_journal_entry(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(entry)))
}

pub async fn delete_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<JournalEntryId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM journal_entry WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}
