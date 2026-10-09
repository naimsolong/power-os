// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, One, Zero};
use power_os_domain::{
    AccountId, BillId, BillLineId, JournalEntryId, JournalLineId, PartyId, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::common::ensure_period_open;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BillStatus {
    Draft,
    Open,
    Partial,
    Paid,
    Overdue,
    Cancelled,
}

impl BillStatus {
    fn as_str(&self) -> &'static str {
        match self {
            BillStatus::Draft => "draft",
            BillStatus::Open => "open",
            BillStatus::Partial => "partial",
            BillStatus::Paid => "paid",
            BillStatus::Overdue => "overdue",
            BillStatus::Cancelled => "cancelled",
        }
    }
}

fn supported_currency(currency: &str) -> bool {
    matches!(currency, "MYR" | "USD" | "EUR" | "SGD")
}

fn normalize_rate(value: Option<&BigDecimal>) -> BigDecimal {
    value.cloned().unwrap_or_else(BigDecimal::one)
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

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBillLineRequest {
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    pub account_id: AccountId,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBillRequest {
    pub party_id: PartyId,
    #[validate(length(min = 1, message = "Bill number is required"))]
    pub bill_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub currency: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
    pub lines: Vec<CreateBillLineRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateBillRequest {
    pub party_id: Option<PartyId>,
    #[validate(length(min = 1, message = "Bill number is required"))]
    pub bill_number: Option<String>,
    pub issue_date: Option<Date>,
    pub due_date: Option<Date>,
    pub status: Option<BillStatus>,
    pub currency: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
    pub lines: Option<Vec<CreateBillLineRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct BillListQuery {
    pub party_id: Option<PartyId>,
    pub status: Option<BillStatus>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BillResponse {
    pub id: BillId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub bill_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub status: String,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub exchange_rate: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct BillDetailResponse {
    pub id: BillId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub bill_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub status: String,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub exchange_rate: BigDecimal,
    pub journal_entry_id: Option<JournalEntryId>,
    pub lines: Vec<BillLineResponse>,
}

#[derive(Debug, Serialize)]
pub struct BillLineResponse {
    pub id: BillLineId,
    pub bill_id: BillId,
    pub description: String,
    pub account_id: AccountId,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub amount: BigDecimal,
    pub foreign_unit_price: BigDecimal,
    pub foreign_amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct PostBillRequest {
    pub payable_account_id: Option<AccountId>,
}

#[derive(Debug, Serialize)]
pub struct PostedBillResponse {
    pub bill: BillDetailResponse,
    pub journal_entry_id: JournalEntryId,
}

fn map_bill_row(row: &sqlx::postgres::PgRow) -> Result<BillResponse, sqlx::Error> {
    Ok(BillResponse {
        id: BillId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: PartyId(row.try_get("party_id")?),
        bill_number: row.try_get("bill_number")?,
        issue_date: row.try_get("issue_date")?,
        due_date: row.try_get("due_date")?,
        status: row.try_get("status")?,
        total_amount: row.try_get("total_amount")?,
        currency: row.try_get("currency")?,
        exchange_rate: row.try_get("exchange_rate")?,
    })
}

fn map_bill_line_row(row: &sqlx::postgres::PgRow) -> Result<BillLineResponse, sqlx::Error> {
    Ok(BillLineResponse {
        id: BillLineId(row.try_get("id")?),
        bill_id: BillId(row.try_get("bill_id")?),
        description: row.try_get("description")?,
        account_id: AccountId(row.try_get("account_id")?),
        quantity: row.try_get("quantity")?,
        unit_price: row.try_get("unit_price")?,
        amount: row.try_get("amount")?,
        foreign_unit_price: row.try_get("foreign_unit_price")?,
        foreign_amount: row.try_get("foreign_amount")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_bills).post(create_bill))
        .route("/{id}", get(get_bill).patch(update_bill))
        .route("/{id}/post", post(post_bill))
        .route("/{id}/cancel", post(cancel_bill))
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

async fn ensure_account_in_workspace(
    db: &mut sqlx::PgConnection,
    account_id: AccountId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    let exists = query("SELECT 1 FROM account WHERE id = $1 AND workspace_id = $2")
        .bind(account_id.0)
        .bind(workspace_id.0)
        .fetch_optional(&mut *db)
        .await?;

    if exists.is_none() {
        return Err(ApiError::BadRequest(
            "Account not found in workspace".to_string(),
        ));
    }
    Ok(())
}

async fn get_or_create_default_account(
    db: &mut sqlx::PgConnection,
    workspace_id: WorkspaceId,
    code: &str,
    name: &str,
    account_type: &str,
) -> Result<AccountId, ApiError> {
    let row = query("SELECT id FROM account WHERE workspace_id = $1 AND code = $2")
        .bind(workspace_id.0)
        .bind(code)
        .fetch_optional(&mut *db)
        .await?;

    if let Some(row) = row {
        let id: uuid::Uuid = row.try_get("id")?;
        return Ok(AccountId::from(id));
    }

    let account_id = AccountId::new();
    query(
        r#"
        INSERT INTO account (id, workspace_id, code, name, account_type)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(account_id.0)
    .bind(workspace_id.0)
    .bind(code)
    .bind(name)
    .bind(account_type)
    .execute(&mut *db)
    .await?;

    Ok(account_id)
}

async fn fetch_bill(
    db: &sqlx::PgPool,
    bill_id: BillId,
    workspace_id: WorkspaceId,
) -> Result<Option<BillResponse>, ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, party_id, bill_number, issue_date, due_date,
               status, total_amount, currency, exchange_rate
        FROM bill
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(bill_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(map_bill_row(&row)?)),
        None => Ok(None),
    }
}

async fn fetch_bill_lines(
    db: &sqlx::PgPool,
    bill_id: BillId,
) -> Result<Vec<BillLineResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, bill_id, description, account_id, quantity, unit_price, amount,
               foreign_unit_price, foreign_amount
        FROM bill_line
        WHERE bill_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(bill_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_bill_line_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

async fn fetch_bill_detail(
    db: &sqlx::PgPool,
    bill_id: BillId,
    workspace_id: WorkspaceId,
) -> Result<Option<BillDetailResponse>, ApiError> {
    let bill = match fetch_bill(db, bill_id, workspace_id).await? {
        Some(b) => b,
        None => return Ok(None),
    };

    let journal_entry_id: Option<JournalEntryId> = query(
        "SELECT journal_entry_id FROM bill WHERE id = $1 AND workspace_id = $2",
    )
    .bind(bill_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?
    .and_then(|row| row.try_get::<Option<uuid::Uuid>, _>("journal_entry_id").ok()?)
    .map(JournalEntryId::from);

    let lines = fetch_bill_lines(db, bill_id).await?;

    Ok(Some(BillDetailResponse {
        id: bill.id,
        workspace_id: bill.workspace_id,
        party_id: bill.party_id,
        bill_number: bill.bill_number,
        issue_date: bill.issue_date,
        due_date: bill.due_date,
        status: bill.status,
        total_amount: bill.total_amount,
        currency: bill.currency,
        exchange_rate: bill.exchange_rate,
        journal_entry_id,
        lines,
    }))
}

async fn insert_bill_lines(
    tx: &mut sqlx::PgConnection,
    bill_id: BillId,
    lines: &[CreateBillLineRequest],
    exchange_rate: &BigDecimal,
) -> Result<BigDecimal, ApiError> {
    let mut total = BigDecimal::zero();

    for line in lines {
        let foreign_amount = &line.quantity * &line.unit_price;
        let myr_amount = &foreign_amount * exchange_rate;
        let myr_unit_price = &line.unit_price * exchange_rate;
        let line_id = BillLineId::new();

        query(
            r#"
            INSERT INTO bill_line (
                id, bill_id, description, account_id, quantity, unit_price, amount,
                foreign_unit_price, foreign_amount
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
        )
        .bind(line_id.0)
        .bind(bill_id.0)
        .bind(&line.description)
        .bind(line.account_id.0)
        .bind(&line.quantity)
        .bind(&myr_unit_price)
        .bind(&myr_amount)
        .bind(&line.unit_price)
        .bind(&foreign_amount)
        .execute(&mut *tx)
        .await?;

        total = total + &myr_amount;
    }

    Ok(total)
}

pub async fn list_bills(
    State(state): State<AppState>,
    Query(params): Query<BillListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<BillResponse>>), ApiError> {
    let status = params.status.as_ref().map(|s| s.as_str().to_string());
    let search = params.search.as_deref().unwrap_or("").trim();

    let rows = query(
        r#"
        SELECT id, workspace_id, party_id, bill_number, issue_date, due_date,
               status, total_amount, currency, exchange_rate
        FROM bill
        WHERE workspace_id = $1
          AND ($2::uuid IS NULL OR party_id = $2)
          AND ($3::text IS NULL OR status = $3)
          AND ($4::text = '' OR bill_number ILIKE '%' || $4 || '%')
        ORDER BY issue_date DESC, bill_number
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.party_id.map(|id| id.0))
    .bind(status)
    .bind(search)
    .fetch_all(&state.db)
    .await?;

    let bills = rows
        .iter()
        .map(map_bill_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(bills)))
}

pub async fn create_bill(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateBillRequest>,
) -> Result<(StatusCode, Json<BillDetailResponse>), ApiError> {
    payload.validate()?;

    ensure_party_in_workspace(&state.db, payload.party_id, auth_user.workspace_id).await?;

    let bill_id = BillId::new();
    let currency = payload.currency.as_deref().unwrap_or("MYR");
    let exchange_rate = normalize_rate(payload.exchange_rate.as_ref());
    validate_currency_and_rate(currency, &exchange_rate)?;

    let mut tx = state.db.begin().await?;

    for line in &payload.lines {
        ensure_account_in_workspace(&mut tx, line.account_id, auth_user.workspace_id).await?;
    }

    query(
        r#"
        INSERT INTO bill (id, workspace_id, party_id, bill_number, issue_date, due_date, status, total_amount, currency, exchange_rate)
        VALUES ($1, $2, $3, $4, $5, $6, 'draft', 0, $7, $8)
        "#,
    )
    .bind(bill_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.0)
    .bind(&payload.bill_number)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(currency)
    .bind(&exchange_rate)
    .execute(&mut *tx)
    .await?;

    let total = insert_bill_lines(&mut tx, bill_id, &payload.lines, &exchange_rate).await?;

    query(
        r#"
        UPDATE bill
        SET total_amount = $3, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(bill_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&total)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let bill = fetch_bill_detail(&state.db, bill_id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(bill)))
}

pub async fn get_bill(
    State(state): State<AppState>,
    Path(id): Path<BillId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<BillDetailResponse>), ApiError> {
    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(bill)))
}

pub async fn update_bill(
    State(state): State<AppState>,
    Path(id): Path<BillId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateBillRequest>,
) -> Result<(StatusCode, Json<BillDetailResponse>), ApiError> {
    payload.validate()?;

    let existing = fetch_bill(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(b) => b,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft bills can be updated".to_string(),
        ));
    }

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    let status = payload.status.as_ref().map(|s| s.as_str());
    let currency = payload.currency.as_deref().unwrap_or(&existing.currency);
    let exchange_rate = payload
        .exchange_rate
        .as_ref()
        .unwrap_or(&existing.exchange_rate);
    validate_currency_and_rate(currency, exchange_rate)?;

    let mut tx = state.db.begin().await?;

    if let Some(lines) = &payload.lines {
        for line in lines {
            ensure_account_in_workspace(&mut tx, line.account_id, auth_user.workspace_id).await?;
        }
    }

    query(
        r#"
        UPDATE bill
        SET
            party_id = COALESCE($3, party_id),
            bill_number = COALESCE($4, bill_number),
            issue_date = COALESCE($5, issue_date),
            due_date = COALESCE($6, due_date),
            status = COALESCE($7, status),
            currency = COALESCE($8, currency),
            exchange_rate = COALESCE($9, exchange_rate),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|p| p.0))
    .bind(payload.bill_number.as_deref())
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(status)
    .bind(payload.currency.as_deref())
    .bind(payload.exchange_rate.as_ref())
    .execute(&mut *tx)
    .await?;

    // Recompute functional (MYR) line amounts when the exchange rate changes and no lines are supplied.
    if payload.lines.is_none()
        && payload.exchange_rate.is_some()
        && payload.exchange_rate.as_ref() != Some(&existing.exchange_rate)
    {
        query(
            r#"
            UPDATE bill_line
            SET unit_price = foreign_unit_price * $2,
                amount = foreign_amount * $2,
                updated_at = now()
            WHERE bill_id = $1
            "#,
        )
        .bind(id.0)
        .bind(exchange_rate)
        .execute(&mut *tx)
        .await?;
    }

    let total = if let Some(lines) = payload.lines {
        query("DELETE FROM bill_line WHERE bill_id = $1")
            .bind(id.0)
            .execute(&mut *tx)
            .await?;
        insert_bill_lines(&mut tx, id, &lines, exchange_rate).await?
    } else {
        existing.total_amount
    };

    query(
        r#"
        UPDATE bill
        SET total_amount = $3, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&total)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(bill)))
}

pub async fn post_bill(
    State(state): State<AppState>,
    Path(id): Path<BillId>,
    auth_user: AuthUser,
    Json(payload): Json<PostBillRequest>,
) -> Result<(StatusCode, Json<PostedBillResponse>), ApiError> {
    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id).await?;
    let bill = match bill {
        Some(b) => b,
        None => return Err(ApiError::NotFound),
    };

    if bill.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft bills can be posted".to_string(),
        ));
    }

    if bill.lines.is_empty() {
        return Err(ApiError::BadRequest(
            "Cannot post a bill without lines".to_string(),
        ));
    }

    let total: BigDecimal = bill
        .lines
        .iter()
        .map(|line| &line.amount)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    let foreign_total: BigDecimal = bill
        .lines
        .iter()
        .map(|line| &line.foreign_amount)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    if total <= BigDecimal::zero() {
        return Err(ApiError::BadRequest(
            "Bill total must be greater than zero".to_string(),
        ));
    }

    ensure_period_open(&state.db, auth_user.workspace_id, bill.issue_date).await?;

    let mut tx = state.db.begin().await?;

    let payable_account_id = match payload.payable_account_id {
        Some(id) => {
            ensure_account_in_workspace(&mut tx, id, auth_user.workspace_id).await?;
            id
        }
        None => {
            get_or_create_default_account(
                &mut tx,
                auth_user.workspace_id,
                "2100",
                "Accounts Payable",
                "liability",
            )
            .await?
        }
    };

    let journal_entry_id = JournalEntryId::new();
    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(bill.issue_date)
    .bind(&bill.bill_number)
    .bind(format!("Bill {}", bill.bill_number))
    .execute(&mut *tx)
    .await?;

    for line in &bill.lines {
        query(
            r#"
            INSERT INTO journal_line (
                id, journal_entry_id, account_id, party_id, description,
                debit, credit, foreign_debit, foreign_credit, exchange_rate, currency
            )
            VALUES ($1, $2, $3, $4, $5, $6, 0, $7, 0, $8, $9)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(journal_entry_id.0)
        .bind(line.account_id.0)
        .bind(bill.party_id.0)
        .bind(format!("{} - Bill {}", line.description, bill.bill_number))
        .bind(&line.amount)
        .bind(&line.foreign_amount)
        .bind(&bill.exchange_rate)
        .bind(&bill.currency)
        .execute(&mut *tx)
        .await?;
    }

    query(
        r#"
        INSERT INTO journal_line (
            id, journal_entry_id, account_id, party_id, description,
            debit, credit, foreign_debit, foreign_credit, exchange_rate, currency
        )
        VALUES ($1, $2, $3, $4, $5, 0, $6, 0, $7, $8, $9)
        "#,
    )
    .bind(JournalLineId::new().0)
    .bind(journal_entry_id.0)
    .bind(payable_account_id.0)
    .bind(bill.party_id.0)
    .bind(format!("Accounts Payable - Bill {}", bill.bill_number))
    .bind(&total)
    .bind(&foreign_total)
    .bind(&bill.exchange_rate)
    .bind(&bill.currency)
    .execute(&mut *tx)
    .await?;

    query(
        r#"
        UPDATE bill
        SET status = 'open', total_amount = $3, journal_entry_id = $4, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&total)
    .bind(journal_entry_id.0)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((
        StatusCode::OK,
        Json(PostedBillResponse {
            bill,
            journal_entry_id,
        }),
    ))
}

pub async fn cancel_bill(
    State(state): State<AppState>,
    Path(id): Path<BillId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<BillDetailResponse>), ApiError> {
    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id).await?;
    let bill = match bill {
        Some(b) => b,
        None => return Err(ApiError::NotFound),
    };

    if bill.status == "draft" || bill.status == "cancelled" {
        return Err(ApiError::BadRequest(
            "Only posted or open bills can be cancelled".to_string(),
        ));
    }

    let original_journal_entry_id = match bill.journal_entry_id {
        Some(id) => id,
        None => return Err(ApiError::BadRequest("Bill has no journal entry".to_string())),
    };

    ensure_period_open(&state.db, auth_user.workspace_id, bill.issue_date).await?;

    let mut tx = state.db.begin().await?;

    let rows = query(
        r#"
        SELECT account_id, party_id, description, debit, credit,
               foreign_debit, foreign_credit, exchange_rate
        FROM journal_line
        WHERE journal_entry_id = $1
        "#,
    )
    .bind(original_journal_entry_id.0)
    .fetch_all(&mut *tx)
    .await?;

    if rows.is_empty() {
        return Err(ApiError::BadRequest(
            "Original journal entry has no lines".to_string(),
        ));
    }

    let reversal_entry_id = JournalEntryId::new();

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(reversal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(bill.issue_date)
    .bind(format!("CNCL-{}", bill.bill_number))
    .bind(format!("Cancellation of bill {}", bill.bill_number))
    .execute(&mut *tx)
    .await?;

    for row in &rows {
        let account_id: AccountId = AccountId(row.try_get::<uuid::Uuid, _>("account_id")?);
        let party_id: Option<PartyId> = row
            .try_get::<Option<uuid::Uuid>, _>("party_id")?
            .map(PartyId::from);
        let description: String = row.try_get("description")?;
        let debit: BigDecimal = row.try_get("debit")?;
        let credit: BigDecimal = row.try_get("credit")?;
        let foreign_debit: BigDecimal = row.try_get("foreign_debit")?;
        let foreign_credit: BigDecimal = row.try_get("foreign_credit")?;
        let exchange_rate: BigDecimal = row.try_get("exchange_rate")?;

        query(
            r#"
            INSERT INTO journal_line (
                id, journal_entry_id, account_id, party_id, description,
                debit, credit, foreign_debit, foreign_credit, exchange_rate
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(reversal_entry_id.0)
        .bind(account_id.0)
        .bind(party_id.map(|p| p.0))
        .bind(format!("{} - reversal", description))
        .bind(credit)
        .bind(debit)
        .bind(foreign_credit)
        .bind(foreign_debit)
        .bind(exchange_rate)
        .execute(&mut *tx)
        .await?;
    }

    query(
        r#"
        UPDATE bill
        SET status = 'cancelled', updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let bill = fetch_bill_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(bill)))
}
