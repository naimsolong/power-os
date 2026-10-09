// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, One, Zero};
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
    pub currency: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateJournalEntryRequest {
    pub entry_date: Date,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub status: Option<JournalEntryStatus>,
    pub lines: Vec<CreateJournalLineRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateJournalEntryRequest {
    pub entry_date: Option<Date>,
    pub reference: Option<String>,
    pub description: Option<String>,
    pub status: Option<JournalEntryStatus>,
    pub lines: Option<Vec<CreateJournalLineRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct JournalEntryListQuery {
    pub status: Option<JournalEntryStatus>,
    pub from: Option<Date>,
    pub to: Option<Date>,
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
    pub foreign_debit: BigDecimal,
    pub foreign_credit: BigDecimal,
    pub exchange_rate: BigDecimal,
    pub currency: String,
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

#[derive(Debug, Serialize)]
pub struct CancelJournalEntryResponse {
    pub original_entry: JournalEntryResponse,
    pub reversing_entry: JournalEntryResponse,
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
        foreign_debit: row.try_get("foreign_debit")?,
        foreign_credit: row.try_get("foreign_credit")?,
        exchange_rate: row.try_get("exchange_rate")?,
        currency: row.try_get("currency")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_journal_entries).post(create_journal_entry))
        .route(
            "/{id}",
            get(get_journal_entry)
                .patch(update_journal_entry)
                .delete(delete_journal_entry),
        )
        .route("/{id}/post", post(post_journal_entry))
        .route("/{id}/cancel", post(cancel_journal_entry))
}

async fn ensure_account_in_workspace(
    db: &sqlx::PgPool,
    account_id: AccountId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists = query(
        "SELECT 1 FROM account WHERE id = $1 AND workspace_id = $2 AND is_active = true",
    )
    .bind(account_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    if exists.is_none() {
        return Err(ApiError::BadRequest(
            "Account not found in workspace or is inactive".to_string(),
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

fn normalize_amount(value: Option<&BigDecimal>) -> BigDecimal {
    value.cloned().unwrap_or_else(BigDecimal::zero)
}

fn normalize_rate(value: Option<&BigDecimal>) -> BigDecimal {
    value.cloned().unwrap_or_else(BigDecimal::one)
}

fn supported_currency(currency: &str) -> bool {
    matches!(currency, "MYR" | "USD" | "EUR" | "SGD")
}

fn validate_currency_and_rate(currency: &str, rate: &BigDecimal) -> Result<(), ApiError> {
    if !supported_currency(currency) {
        return Err(ApiError::BadRequest(format!(
            "Unsupported currency: {}. Supported: MYR, USD, EUR, SGD",
            currency
        )));
    }
    if rate <= &BigDecimal::zero() {
        return Err(ApiError::BadRequest(
            "Exchange rate must be greater than zero".to_string(),
        ));
    }
    if currency == "MYR" && rate != &BigDecimal::one() {
        return Err(ApiError::BadRequest(
            "MYR transactions must use exchange rate 1".to_string(),
        ));
    }
    Ok(())
}

fn compute_myr_amount(foreign: &BigDecimal, rate: &BigDecimal) -> BigDecimal {
    foreign * rate
}

fn validate_request_lines(
    lines: &[CreateJournalLineRequest],
) -> Result<(BigDecimal, BigDecimal, BigDecimal, BigDecimal), ApiError> {
    if lines.len() < 2 {
        return Err(ApiError::BadRequest(
            "At least two journal lines are required".to_string(),
        ));
    }

    let zero = BigDecimal::zero();
    let mut total_foreign_debit = BigDecimal::zero();
    let mut total_foreign_credit = BigDecimal::zero();
    let mut total_myr_debit = BigDecimal::zero();
    let mut total_myr_credit = BigDecimal::zero();

    for (idx, line) in lines.iter().enumerate() {
        let currency = line.currency.as_deref().unwrap_or("MYR");
        let rate = normalize_rate(line.exchange_rate.as_ref());
        validate_currency_and_rate(currency, &rate)?;

        let foreign_debit = normalize_amount(line.debit.as_ref());
        let foreign_credit = normalize_amount(line.credit.as_ref());

        if foreign_debit < zero || foreign_credit < zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: debit and credit must be non-negative",
                idx + 1
            )));
        }

        if foreign_debit > zero && foreign_credit > zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: a line cannot have both debit and credit",
                idx + 1
            )));
        }

        if foreign_debit == zero && foreign_credit == zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: a line must have either debit or credit greater than zero",
                idx + 1
            )));
        }

        let myr_debit = compute_myr_amount(&foreign_debit, &rate);
        let myr_credit = compute_myr_amount(&foreign_credit, &rate);

        total_foreign_debit += foreign_debit;
        total_foreign_credit += foreign_credit;
        total_myr_debit += myr_debit;
        total_myr_credit += myr_credit;
    }

    if total_myr_debit != total_myr_credit {
        return Err(ApiError::BadRequest(
            "Journal entry debits must equal credits".to_string(),
        ));
    }

    Ok((
        total_foreign_debit,
        total_foreign_credit,
        total_myr_debit,
        total_myr_credit,
    ))
}

fn validate_myr_balance(lines: &[JournalLineResponse]) -> Result<(), ApiError> {
    if lines.len() < 2 {
        return Err(ApiError::BadRequest(
            "At least two journal lines are required".to_string(),
        ));
    }

    let zero = BigDecimal::zero();
    let mut total_debit = BigDecimal::zero();
    let mut total_credit = BigDecimal::zero();

    for (idx, line) in lines.iter().enumerate() {
        if line.debit < zero || line.credit < zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: debit and credit must be non-negative",
                idx + 1
            )));
        }

        if line.debit > zero && line.credit > zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: a line cannot have both debit and credit",
                idx + 1
            )));
        }

        if line.debit == zero && line.credit == zero {
            return Err(ApiError::BadRequest(format!(
                "Line {}: a line must have either debit or credit greater than zero",
                idx + 1
            )));
        }

        total_debit += &line.debit;
        total_credit += &line.credit;
    }

    if total_debit != total_credit {
        return Err(ApiError::BadRequest(
            "Journal entry debits must equal credits".to_string(),
        ));
    }

    Ok(())
}

async fn validate_lines_accounts(
    db: &sqlx::PgPool,
    lines: &[CreateJournalLineRequest],
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    for line in lines {
        ensure_account_in_workspace(db, line.account_id, workspace_id).await?;
        if let Some(party_id) = line.party_id {
            ensure_party_in_workspace(db, party_id, workspace_id).await?;
        }
    }
    Ok(())
}

async fn insert_journal_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    journal_entry_id: JournalEntryId,
    lines: &[CreateJournalLineRequest],
) -> Result<(), ApiError> {
    for line in lines {
        let currency = line.currency.as_deref().unwrap_or("MYR");
        let rate = normalize_rate(line.exchange_rate.as_ref());
        validate_currency_and_rate(currency, &rate)?;

        let foreign_debit = normalize_amount(line.debit.as_ref());
        let foreign_credit = normalize_amount(line.credit.as_ref());
        let myr_debit = compute_myr_amount(&foreign_debit, &rate);
        let myr_credit = compute_myr_amount(&foreign_credit, &rate);

        query(
            r#"
            INSERT INTO journal_line (
                id, journal_entry_id, account_id, party_id, description,
                debit, credit, foreign_debit, foreign_credit, exchange_rate, currency
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(journal_entry_id.0)
        .bind(line.account_id.0)
        .bind(line.party_id.map(|p| p.0))
        .bind(line.description.as_deref())
        .bind(&myr_debit)
        .bind(&myr_credit)
        .bind(&foreign_debit)
        .bind(&foreign_credit)
        .bind(&rate)
        .bind(currency)
        .execute(&mut **tx)
        .await?;
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
        SELECT id, journal_entry_id, account_id, party_id, description, debit, credit,
               foreign_debit, foreign_credit, exchange_rate, currency
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
          AND ($3::date IS NULL OR entry_date >= $3)
          AND ($4::date IS NULL OR entry_date <= $4)
        ORDER BY entry_date DESC, reference
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(status)
    .bind(params.from)
    .bind(params.to)
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

    validate_request_lines(&payload.lines)?;
    validate_lines_accounts(&state.db, &payload.lines, auth_user.workspace_id).await?;

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

    insert_journal_lines(&mut tx, journal_entry_id, &payload.lines).await?;

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

pub async fn update_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<JournalEntryId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateJournalEntryRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), ApiError> {
    payload.validate()?;

    let existing = fetch_journal_entry(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(entry) => entry,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft journal entries can be updated".to_string(),
        ));
    }

    if let Some(ref lines) = payload.lines {
        validate_request_lines(lines)?;
        validate_lines_accounts(&state.db, lines, auth_user.workspace_id).await?;
    }

    let status = payload.status.as_ref().map(|s| s.as_str());

    let mut tx = state.db.begin().await?;

    query(
        r#"
        UPDATE journal_entry
        SET
            entry_date = COALESCE($3, entry_date),
            reference = COALESCE($4, reference),
            description = COALESCE($5, description),
            status = COALESCE($6, status),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.entry_date)
    .bind(payload.reference.as_deref())
    .bind(payload.description.as_deref())
    .bind(status)
    .execute(&mut *tx)
    .await?;

    if let Some(lines) = payload.lines {
        query("DELETE FROM journal_line WHERE journal_entry_id = $1")
            .bind(id.0)
            .execute(&mut *tx)
            .await?;

        insert_journal_lines(&mut tx, id, &lines).await?;
    }

    tx.commit().await?;

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
    let existing = fetch_journal_entry(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(entry) => entry,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft journal entries can be deleted".to_string(),
        ));
    }

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

pub async fn post_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<JournalEntryId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<JournalEntryResponse>), ApiError> {
    let existing = fetch_journal_entry(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(entry) => entry,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft journal entries can be posted".to_string(),
        ));
    }

    validate_myr_balance(&existing.lines)?;

    query(
        r#"
        UPDATE journal_entry
        SET status = 'posted', updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&state.db)
    .await?;

    let entry = fetch_journal_entry(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(entry)))
}

pub async fn cancel_journal_entry(
    State(state): State<AppState>,
    Path(id): Path<JournalEntryId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<CancelJournalEntryResponse>), ApiError> {
    let original = fetch_journal_entry(&state.db, id, auth_user.workspace_id).await?;
    let original = match original {
        Some(entry) => entry,
        None => return Err(ApiError::NotFound),
    };

    if original.status != "posted" {
        return Err(ApiError::BadRequest(
            "Only posted journal entries can be cancelled".to_string(),
        ));
    }

    let reversing_entry_id = JournalEntryId::new();
    let reference = original
        .reference
        .as_deref()
        .map(|r| format!("REV-{}", r))
        .or_else(|| Some(format!("REV-{}", id.0)));
    let description = original
        .description
        .as_deref()
        .map(|d| format!("Reversal of {}", d))
        .or_else(|| Some(format!("Reversal of journal entry {}", id.0)));

    let mut tx = state.db.begin().await?;

    query(
        r#"
        UPDATE journal_entry
        SET status = 'cancelled', updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&mut *tx)
    .await?;

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(reversing_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(original.entry_date)
    .bind(&reference)
    .bind(&description)
    .execute(&mut *tx)
    .await?;

    for line in &original.lines {
        query(
            r#"
            INSERT INTO journal_line (
                id, journal_entry_id, account_id, party_id, description,
                debit, credit, foreign_debit, foreign_credit, exchange_rate, currency
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(reversing_entry_id.0)
        .bind(line.account_id.0)
        .bind(line.party_id.map(|p| p.0))
        .bind(line.description.as_deref().map(|d| format!("Reversal - {}", d)))
        .bind(&line.credit)
        .bind(&line.debit)
        .bind(&line.foreign_credit)
        .bind(&line.foreign_debit)
        .bind(&line.exchange_rate)
        .bind(&line.currency)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let original = fetch_journal_entry(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;
    let reversing = fetch_journal_entry(&state.db, reversing_entry_id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::Internal)?;

    Ok((
        StatusCode::OK,
        Json(CancelJournalEntryResponse {
            original_entry: original,
            reversing_entry: reversing,
        }),
    ))
}
