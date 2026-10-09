// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, Zero};
use power_os_domain::{
    AccountId, BillId, InvoiceId, JournalEntryId, JournalLineId, PartyId, PaymentAllocationId,
    PaymentId, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentDirection {
    Received,
    Sent,
}

impl PaymentDirection {
    fn as_str(&self) -> &'static str {
        match self {
            PaymentDirection::Received => "received",
            PaymentDirection::Sent => "sent",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    BankTransfer,
    Cash,
    Cheque,
    Card,
}

impl PaymentMethod {
    fn as_str(&self) -> &'static str {
        match self {
            PaymentMethod::BankTransfer => "bank_transfer",
            PaymentMethod::Cash => "cash",
            PaymentMethod::Cheque => "cheque",
            PaymentMethod::Card => "card",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Draft,
    Posted,
    Cancelled,
}

impl PaymentStatus {
    fn as_str(&self) -> &'static str {
        match self {
            PaymentStatus::Draft => "draft",
            PaymentStatus::Posted => "posted",
            PaymentStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentAllocationRequest {
    pub invoice_id: Option<InvoiceId>,
    pub bill_id: Option<BillId>,
    pub amount: BigDecimal,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreatePaymentRequest {
    pub party_id: PartyId,
    pub bank_account_id: AccountId,
    pub payment_date: Date,
    pub amount: BigDecimal,
    pub currency: Option<String>,
    pub payment_method: PaymentMethod,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub direction: PaymentDirection,
    pub allocations: Vec<CreatePaymentAllocationRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePaymentRequest {
    pub party_id: Option<PartyId>,
    pub bank_account_id: Option<AccountId>,
    pub payment_date: Option<Date>,
    pub amount: Option<BigDecimal>,
    pub currency: Option<String>,
    pub payment_method: Option<PaymentMethod>,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub direction: Option<PaymentDirection>,
    pub allocations: Option<Vec<CreatePaymentAllocationRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct PaymentListQuery {
    pub party_id: Option<PartyId>,
    pub direction: Option<PaymentDirection>,
    pub status: Option<PaymentStatus>,
}

#[derive(Debug, Serialize)]
pub struct PaymentResponse {
    pub id: PaymentId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub bank_account_id: AccountId,
    pub payment_date: Date,
    pub amount: BigDecimal,
    pub currency: String,
    pub payment_method: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub direction: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct PaymentAllocationResponse {
    pub id: PaymentAllocationId,
    pub payment_id: PaymentId,
    pub invoice_id: Option<InvoiceId>,
    pub bill_id: Option<BillId>,
    pub amount: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct PaymentDetailResponse {
    pub id: PaymentId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub bank_account_id: AccountId,
    pub payment_date: Date,
    pub amount: BigDecimal,
    pub currency: String,
    pub payment_method: String,
    pub reference: Option<String>,
    pub notes: Option<String>,
    pub direction: String,
    pub status: String,
    pub journal_entry_id: Option<JournalEntryId>,
    pub allocations: Vec<PaymentAllocationResponse>,
}

fn map_payment_row(row: &sqlx::postgres::PgRow) -> Result<PaymentResponse, sqlx::Error> {
    Ok(PaymentResponse {
        id: PaymentId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: PartyId(row.try_get("party_id")?),
        bank_account_id: AccountId(row.try_get("bank_account_id")?),
        payment_date: row.try_get("payment_date")?,
        amount: row.try_get("amount")?,
        currency: row.try_get("currency")?,
        payment_method: row.try_get("payment_method")?,
        reference: row.try_get("reference")?,
        notes: row.try_get("notes")?,
        direction: row.try_get("direction")?,
        status: row.try_get("status")?,
    })
}

fn map_payment_allocation_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PaymentAllocationResponse, sqlx::Error> {
    Ok(PaymentAllocationResponse {
        id: PaymentAllocationId(row.try_get("id")?),
        payment_id: PaymentId(row.try_get("payment_id")?),
        invoice_id: row
            .try_get::<Option<uuid::Uuid>, _>("invoice_id")?
            .map(InvoiceId::from),
        bill_id: row
            .try_get::<Option<uuid::Uuid>, _>("bill_id")?
            .map(BillId::from),
        amount: row.try_get("amount")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_payments).post(create_payment))
        .route("/{id}", get(get_payment).patch(update_payment))
        .route("/{id}/post", post(post_payment))
        .route("/{id}/cancel", post(cancel_payment))
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

async fn fetch_payment(
    db: &sqlx::PgPool,
    payment_id: PaymentId,
    workspace_id: WorkspaceId,
) -> Result<Option<PaymentResponse>, ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, party_id, bank_account_id, payment_date,
               amount, currency, payment_method, reference, notes,
               direction, status
        FROM payment
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(payment_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(map_payment_row(&row)?)),
        None => Ok(None),
    }
}

async fn fetch_payment_allocations(
    db: &sqlx::PgPool,
    payment_id: PaymentId,
) -> Result<Vec<PaymentAllocationResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, payment_id, invoice_id, bill_id, amount
        FROM payment_allocation
        WHERE payment_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(payment_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_payment_allocation_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

async fn fetch_payment_detail(
    db: &sqlx::PgPool,
    payment_id: PaymentId,
    workspace_id: WorkspaceId,
) -> Result<Option<PaymentDetailResponse>, ApiError> {
    let payment = match fetch_payment(db, payment_id, workspace_id).await? {
        Some(p) => p,
        None => return Ok(None),
    };

    let journal_entry_id: Option<JournalEntryId> = query(
        "SELECT journal_entry_id FROM payment WHERE id = $1 AND workspace_id = $2",
    )
    .bind(payment_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?
    .and_then(|row| row.try_get::<Option<uuid::Uuid>, _>("journal_entry_id").ok()?)
    .map(JournalEntryId::from);

    let allocations = fetch_payment_allocations(db, payment_id).await?;

    Ok(Some(PaymentDetailResponse {
        id: payment.id,
        workspace_id: payment.workspace_id,
        party_id: payment.party_id,
        bank_account_id: payment.bank_account_id,
        payment_date: payment.payment_date,
        amount: payment.amount,
        currency: payment.currency,
        payment_method: payment.payment_method,
        reference: payment.reference,
        notes: payment.notes,
        direction: payment.direction,
        status: payment.status,
        journal_entry_id,
        allocations,
    }))
}

fn validate_allocations(
    direction: &PaymentDirection,
    allocations: &[CreatePaymentAllocationRequest],
) -> Result<(), ApiError> {
    if allocations.is_empty() {
        return Err(ApiError::BadRequest(
            "At least one allocation is required".to_string(),
        ));
    }

    for allocation in allocations {
        if allocation.amount <= BigDecimal::zero() {
            return Err(ApiError::BadRequest(
                "Allocation amount must be greater than zero".to_string(),
            ));
        }

        match direction {
            PaymentDirection::Received => {
                if allocation.invoice_id.is_none() {
                    return Err(ApiError::BadRequest(
                        "Received payments must be allocated to invoices".to_string(),
                    ));
                }
                if allocation.bill_id.is_some() {
                    return Err(ApiError::BadRequest(
                        "Received payments cannot be allocated to bills".to_string(),
                    ));
                }
            }
            PaymentDirection::Sent => {
                if allocation.bill_id.is_none() {
                    return Err(ApiError::BadRequest(
                        "Sent payments must be allocated to bills".to_string(),
                    ));
                }
                if allocation.invoice_id.is_some() {
                    return Err(ApiError::BadRequest(
                        "Sent payments cannot be allocated to invoices".to_string(),
                    ));
                }
            }
        }
    }

    Ok(())
}

async fn validate_allocation_targets(
    tx: &mut sqlx::PgConnection,
    allocations: &[CreatePaymentAllocationRequest],
    party_id: PartyId,
    workspace_id: WorkspaceId,
) -> Result<(), ApiError> {
    for allocation in allocations {
        if let Some(invoice_id) = allocation.invoice_id {
            let row = query(
                "SELECT party_id, total_amount FROM invoice WHERE id = $1 AND workspace_id = $2",
            )
            .bind(invoice_id.0)
            .bind(workspace_id.0)
            .fetch_optional(&mut *tx)
            .await?;

            let (invoice_party_id, _): (uuid::Uuid, BigDecimal) = match row {
                Some(row) => (row.try_get("party_id")?, row.try_get("total_amount")?),
                None => {
                    return Err(ApiError::BadRequest(
                        "Invoice not found in workspace".to_string(),
                    ))
                }
            };

            if invoice_party_id != party_id.0 {
                return Err(ApiError::BadRequest(
                    "Invoice does not belong to selected party".to_string(),
                ));
            }
        }

        if let Some(bill_id) = allocation.bill_id {
            let row = query(
                "SELECT party_id, total_amount FROM bill WHERE id = $1 AND workspace_id = $2",
            )
            .bind(bill_id.0)
            .bind(workspace_id.0)
            .fetch_optional(&mut *tx)
            .await?;

            let (bill_party_id, _): (uuid::Uuid, BigDecimal) = match row {
                Some(row) => (row.try_get("party_id")?, row.try_get("total_amount")?),
                None => {
                    return Err(ApiError::BadRequest(
                        "Bill not found in workspace".to_string(),
                    ))
                }
            };

            if bill_party_id != party_id.0 {
                return Err(ApiError::BadRequest(
                    "Bill does not belong to selected party".to_string(),
                ));
            }
        }
    }

    Ok(())
}

async fn insert_payment_allocations(
    tx: &mut sqlx::PgConnection,
    payment_id: PaymentId,
    allocations: &[CreatePaymentAllocationRequest],
) -> Result<BigDecimal, ApiError> {
    let mut total = BigDecimal::zero();

    for allocation in allocations {
        let allocation_id = PaymentAllocationId::new();

        query(
            r#"
            INSERT INTO payment_allocation (id, payment_id, invoice_id, bill_id, amount)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(allocation_id.0)
        .bind(payment_id.0)
        .bind(allocation.invoice_id.map(|id| id.0))
        .bind(allocation.bill_id.map(|id| id.0))
        .bind(&allocation.amount)
        .execute(&mut *tx)
        .await?;

        total = total + &allocation.amount;
    }

    Ok(total)
}

async fn get_allocated_total(
    tx: &mut sqlx::PgConnection,
    invoice_id: Option<InvoiceId>,
    bill_id: Option<BillId>,
) -> Result<BigDecimal, ApiError> {
    let (sql, id) = if let Some(invoice_id) = invoice_id {
        (
            r#"
            SELECT COALESCE(SUM(amount), 0) AS total
            FROM payment_allocation pa
            JOIN payment p ON p.id = pa.payment_id
            WHERE pa.invoice_id = $1 AND p.status = 'posted'
            "#,
            invoice_id.0,
        )
    } else if let Some(bill_id) = bill_id {
        (
            r#"
            SELECT COALESCE(SUM(amount), 0) AS total
            FROM payment_allocation pa
            JOIN payment p ON p.id = pa.payment_id
            WHERE pa.bill_id = $1 AND p.status = 'posted'
            "#,
            bill_id.0,
        )
    } else {
        return Ok(BigDecimal::zero());
    };

    let row = query(&sql).bind(id).fetch_one(&mut *tx).await?;
    Ok(row.try_get("total")?)
}

async fn update_invoice_status(
    tx: &mut sqlx::PgConnection,
    invoice_id: InvoiceId,
) -> Result<(), ApiError> {
    let invoice = query(
        "SELECT total_amount, status FROM invoice WHERE id = $1",
    )
    .bind(invoice_id.0)
    .fetch_one(&mut *tx)
    .await?;

    let total_amount: BigDecimal = invoice.try_get("total_amount")?;
    let status: String = invoice.try_get("status")?;

    if status == "cancelled" || status == "draft" {
        return Ok(());
    }

    let allocated: BigDecimal = get_allocated_total(tx, Some(invoice_id), None).await?;

    let new_status = if allocated >= total_amount {
        "paid"
    } else if allocated > BigDecimal::zero() {
        "partial"
    } else {
        "posted"
    };

    query(
        "UPDATE invoice SET status = $2, updated_at = now() WHERE id = $1",
    )
    .bind(invoice_id.0)
    .bind(new_status)
    .execute(&mut *tx)
    .await?;

    Ok(())
}

async fn update_bill_status(
    tx: &mut sqlx::PgConnection,
    bill_id: BillId,
) -> Result<(), ApiError> {
    let bill = query("SELECT total_amount, status FROM bill WHERE id = $1")
        .bind(bill_id.0)
        .fetch_one(&mut *tx)
        .await?;

    let total_amount: BigDecimal = bill.try_get("total_amount")?;
    let status: String = bill.try_get("status")?;

    if status == "cancelled" || status == "draft" {
        return Ok(());
    }

    let allocated: BigDecimal = get_allocated_total(tx, None, Some(bill_id)).await?;

    let new_status = if allocated >= total_amount {
        "paid"
    } else if allocated > BigDecimal::zero() {
        "partial"
    } else {
        "open"
    };

    query("UPDATE bill SET status = $2, updated_at = now() WHERE id = $1")
        .bind(bill_id.0)
        .bind(new_status)
        .execute(&mut *tx)
        .await?;

    Ok(())
}

pub async fn list_payments(
    State(state): State<AppState>,
    Query(params): Query<PaymentListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<PaymentResponse>>), ApiError> {
    let direction = params.direction.as_ref().map(|d| d.as_str().to_string());
    let status = params.status.as_ref().map(|s| s.as_str().to_string());

    let rows = query(
        r#"
        SELECT id, workspace_id, party_id, bank_account_id, payment_date,
               amount, currency, payment_method, reference, notes,
               direction, status
        FROM payment
        WHERE workspace_id = $1
          AND ($2::uuid IS NULL OR party_id = $2)
          AND ($3::text IS NULL OR direction = $3)
          AND ($4::text IS NULL OR status = $4)
        ORDER BY payment_date DESC, created_at DESC
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.party_id.map(|id| id.0))
    .bind(direction)
    .bind(status)
    .fetch_all(&state.db)
    .await?;

    let payments = rows
        .iter()
        .map(map_payment_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(payments)))
}

pub async fn create_payment(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<(StatusCode, Json<PaymentDetailResponse>), ApiError> {
    payload.validate()?;

    ensure_party_in_workspace(&state.db, payload.party_id, auth_user.workspace_id).await?;
    validate_allocations(&payload.direction, &payload.allocations)?;

    if payload.amount <= BigDecimal::zero() {
        return Err(ApiError::BadRequest(
            "Payment amount must be greater than zero".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    ensure_account_in_workspace(&mut tx, payload.bank_account_id, auth_user.workspace_id).await?;
    validate_allocation_targets(
        &mut tx,
        &payload.allocations,
        payload.party_id,
        auth_user.workspace_id,
    )
    .await?;

    let currency = payload.currency.as_deref().unwrap_or("MYR");
    let payment_id = PaymentId::new();

    query(
        r#"
        INSERT INTO payment (
            id, workspace_id, party_id, bank_account_id, payment_date,
            amount, currency, payment_method, reference, notes,
            direction, status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'draft')
        "#,
    )
    .bind(payment_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.0)
    .bind(payload.bank_account_id.0)
    .bind(payload.payment_date)
    .bind(&payload.amount)
    .bind(currency)
    .bind(payload.payment_method.as_str())
    .bind(payload.reference.as_deref())
    .bind(payload.notes.as_deref())
    .bind(payload.direction.as_str())
    .execute(&mut *tx)
    .await?;

    insert_payment_allocations(&mut tx, payment_id, &payload.allocations).await?;

    tx.commit().await?;

    let payment = fetch_payment_detail(&state.db, payment_id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(payment)))
}

pub async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<PaymentId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<PaymentDetailResponse>), ApiError> {
    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(payment)))
}

pub async fn update_payment(
    State(state): State<AppState>,
    Path(id): Path<PaymentId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdatePaymentRequest>,
) -> Result<(StatusCode, Json<PaymentDetailResponse>), ApiError> {
    payload.validate()?;

    let existing = fetch_payment(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(p) => p,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft payments can be updated".to_string(),
        ));
    }

    let direction = payload.direction.as_ref().unwrap_or_else(|| {
        match existing.direction.as_str() {
            "sent" => &PaymentDirection::Sent,
            _ => &PaymentDirection::Received,
        }
    });

    if let Some(allocations) = &payload.allocations {
        validate_allocations(direction, allocations)?;
    }

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    let mut tx = state.db.begin().await?;

    if let Some(bank_account_id) = payload.bank_account_id {
        ensure_account_in_workspace(&mut tx, bank_account_id, auth_user.workspace_id).await?;
    }

    if let Some(allocations) = &payload.allocations {
        validate_allocation_targets(
            &mut tx,
            allocations,
            payload.party_id.unwrap_or(existing.party_id),
            auth_user.workspace_id,
        )
        .await?;
    }

    query(
        r#"
        UPDATE payment
        SET
            party_id = COALESCE($3, party_id),
            bank_account_id = COALESCE($4, bank_account_id),
            payment_date = COALESCE($5, payment_date),
            amount = COALESCE($6, amount),
            currency = COALESCE($7, currency),
            payment_method = COALESCE($8, payment_method),
            reference = COALESCE($9, reference),
            notes = COALESCE($10, notes),
            direction = COALESCE($11, direction),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|p| p.0))
    .bind(payload.bank_account_id.map(|a| a.0))
    .bind(payload.payment_date)
    .bind(payload.amount.as_ref())
    .bind(payload.currency.as_deref())
    .bind(payload.payment_method.as_ref().map(|m| m.as_str()))
    .bind(payload.reference.as_deref())
    .bind(payload.notes.as_deref())
    .bind(payload.direction.as_ref().map(|d| d.as_str()))
    .execute(&mut *tx)
    .await?;

    if let Some(allocations) = payload.allocations {
        query("DELETE FROM payment_allocation WHERE payment_id = $1")
            .bind(id.0)
            .execute(&mut *tx)
            .await?;
        insert_payment_allocations(&mut tx, id, &allocations).await?;
    }

    tx.commit().await?;

    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(payment)))
}

pub async fn post_payment(
    State(state): State<AppState>,
    Path(id): Path<PaymentId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<PaymentDetailResponse>), ApiError> {
    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id).await?;
    let payment = match payment {
        Some(p) => p,
        None => return Err(ApiError::NotFound),
    };

    if payment.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft payments can be posted".to_string(),
        ));
    }

    if payment.allocations.is_empty() {
        return Err(ApiError::BadRequest(
            "Cannot post a payment without allocations".to_string(),
        ));
    }

    let allocation_total: BigDecimal = payment
        .allocations
        .iter()
        .map(|a| &a.amount)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    if allocation_total > payment.amount {
        return Err(ApiError::BadRequest(
            "Total allocations cannot exceed payment amount".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    ensure_account_in_workspace(&mut tx, payment.bank_account_id, auth_user.workspace_id).await?;

    let (counterparty_account_id, prepayment_account_id, prepayment_name) =
        match payment.direction.as_str() {
            "received" => {
                let ar_id = get_or_create_default_account(
                    &mut tx,
                    auth_user.workspace_id,
                    "1200",
                    "Accounts Receivable",
                    "asset",
                )
                .await?;
                let prepay_id = get_or_create_default_account(
                    &mut tx,
                    auth_user.workspace_id,
                    "2105",
                    "Customer Prepayments",
                    "liability",
                )
                .await?;
                (ar_id, prepay_id, "Customer Prepayments")
            }
            _ => {
                let ap_id = get_or_create_default_account(
                    &mut tx,
                    auth_user.workspace_id,
                    "2100",
                    "Accounts Payable",
                    "liability",
                )
                .await?;
                let prepay_id = get_or_create_default_account(
                    &mut tx,
                    auth_user.workspace_id,
                    "1205",
                    "Vendor Prepayments",
                    "asset",
                )
                .await?;
                (ap_id, prepay_id, "Vendor Prepayments")
            }
        };

    let remainder = &payment.amount - &allocation_total;

    let journal_entry_id = JournalEntryId::new();
    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payment.payment_date)
    .bind(payment.reference.as_deref().unwrap_or(""))
    .bind(format!("Payment {}", payment.reference.as_deref().unwrap_or("")))
    .execute(&mut *tx)
    .await?;

    match payment.direction.as_str() {
        "received" => {
            query(
                r#"
                INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                VALUES ($1, $2, $3, $4, $5, $6, 0)
                "#,
            )
            .bind(JournalLineId::new().0)
            .bind(journal_entry_id.0)
            .bind(payment.bank_account_id.0)
            .bind(payment.party_id.0)
            .bind(format!("Bank - Payment {}", payment.reference.as_deref().unwrap_or("")))
            .bind(&payment.amount)
            .execute(&mut *tx)
            .await?;

            query(
                r#"
                INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                VALUES ($1, $2, $3, $4, $5, 0, $6)
                "#,
            )
            .bind(JournalLineId::new().0)
            .bind(journal_entry_id.0)
            .bind(counterparty_account_id.0)
            .bind(payment.party_id.0)
            .bind(format!(
                "Accounts Receivable - Payment {}",
                payment.reference.as_deref().unwrap_or("")
            ))
            .bind(&allocation_total)
            .execute(&mut *tx)
            .await?;

            if remainder > BigDecimal::zero() {
                query(
                    r#"
                    INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                    VALUES ($1, $2, $3, $4, $5, 0, $6)
                    "#,
                )
                .bind(JournalLineId::new().0)
                .bind(journal_entry_id.0)
                .bind(prepayment_account_id.0)
                .bind(payment.party_id.0)
                .bind(format!(
                    "{} - Payment {}",
                    prepayment_name,
                    payment.reference.as_deref().unwrap_or("")
                ))
                .bind(&remainder)
                .execute(&mut *tx)
                .await?;
            }
        }
        _ => {
            query(
                r#"
                INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                VALUES ($1, $2, $3, $4, $5, $6, 0)
                "#,
            )
            .bind(JournalLineId::new().0)
            .bind(journal_entry_id.0)
            .bind(counterparty_account_id.0)
            .bind(payment.party_id.0)
            .bind(format!(
                "Accounts Payable - Payment {}",
                payment.reference.as_deref().unwrap_or("")
            ))
            .bind(&allocation_total)
            .execute(&mut *tx)
            .await?;

            if remainder > BigDecimal::zero() {
                query(
                    r#"
                    INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                    VALUES ($1, $2, $3, $4, $5, $6, 0)
                    "#,
                )
                .bind(JournalLineId::new().0)
                .bind(journal_entry_id.0)
                .bind(prepayment_account_id.0)
                .bind(payment.party_id.0)
                .bind(format!(
                    "{} - Payment {}",
                    prepayment_name,
                    payment.reference.as_deref().unwrap_or("")
                ))
                .bind(&remainder)
                .execute(&mut *tx)
                .await?;
            }

            query(
                r#"
                INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
                VALUES ($1, $2, $3, $4, $5, 0, $6)
                "#,
            )
            .bind(JournalLineId::new().0)
            .bind(journal_entry_id.0)
            .bind(payment.bank_account_id.0)
            .bind(payment.party_id.0)
            .bind(format!("Bank - Payment {}", payment.reference.as_deref().unwrap_or("")))
            .bind(&payment.amount)
            .execute(&mut *tx)
            .await?;
        }
    }

    query(
        r#"
        UPDATE payment
        SET status = 'posted', journal_entry_id = $3, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(journal_entry_id.0)
    .execute(&mut *tx)
    .await?;

    for allocation in &payment.allocations {
        if let Some(invoice_id) = allocation.invoice_id {
            update_invoice_status(&mut tx, invoice_id).await?;
        }
        if let Some(bill_id) = allocation.bill_id {
            update_bill_status(&mut tx, bill_id).await?;
        }
    }

    tx.commit().await?;

    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(payment)))
}

pub async fn cancel_payment(
    State(state): State<AppState>,
    Path(id): Path<PaymentId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<PaymentDetailResponse>), ApiError> {
    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id).await?;
    let payment = match payment {
        Some(p) => p,
        None => return Err(ApiError::NotFound),
    };

    if payment.status != "posted" {
        return Err(ApiError::BadRequest(
            "Only posted payments can be cancelled".to_string(),
        ));
    }

    let original_journal_entry_id = match payment.journal_entry_id {
        Some(id) => id,
        None => return Err(ApiError::BadRequest("Payment has no journal entry".to_string())),
    };

    let mut tx = state.db.begin().await?;

    let rows = query(
        r#"
        SELECT account_id, party_id, description, debit, credit
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
    let today = time::OffsetDateTime::now_utc().date();

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(reversal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(today)
    .bind(format!("CNCL-{}", payment.reference.as_deref().unwrap_or("")))
    .bind(format!("Cancellation of payment {}", payment.reference.as_deref().unwrap_or("")))
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

        query(
            r#"
            INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(reversal_entry_id.0)
        .bind(account_id.0)
        .bind(party_id.map(|p| p.0))
        .bind(format!("{} - reversal", description))
        .bind(credit)
        .bind(debit)
        .execute(&mut *tx)
        .await?;
    }

    query(
        r#"
        UPDATE payment
        SET status = 'cancelled', updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&mut *tx)
    .await?;

    for allocation in &payment.allocations {
        if let Some(invoice_id) = allocation.invoice_id {
            update_invoice_status(&mut tx, invoice_id).await?;
        }
        if let Some(bill_id) = allocation.bill_id {
            update_bill_status(&mut tx, bill_id).await?;
        }
    }

    tx.commit().await?;

    let payment = fetch_payment_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(payment)))
}
