// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{get, patch, post},
    Json, Router,
};
use bigdecimal::{BigDecimal, One, Zero};
use power_os_domain::{
    AccountId, InvoiceId, InvoiceLineId, JournalEntryId, JournalLineId, PartyId, WorkspaceId,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;
use validator::Validate;

use crate::auth::AuthUser;
use crate::lhdn;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Sent,
    Paid,
    Overdue,
    Cancelled,
    Posted,
}

impl InvoiceStatus {
    fn as_str(&self) -> &'static str {
        match self {
            InvoiceStatus::Draft => "draft",
            InvoiceStatus::Sent => "sent",
            InvoiceStatus::Paid => "paid",
            InvoiceStatus::Overdue => "overdue",
            InvoiceStatus::Cancelled => "cancelled",
            InvoiceStatus::Posted => "posted",
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
pub struct CreateInvoiceRequest {
    pub party_id: PartyId,
    #[validate(length(min = 1, message = "Invoice number is required"))]
    pub invoice_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub currency: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInvoiceRequest {
    pub party_id: Option<PartyId>,
    #[validate(length(min = 1, message = "Invoice number is required"))]
    pub invoice_number: Option<String>,
    pub issue_date: Option<Date>,
    pub due_date: Option<Date>,
    pub status: Option<InvoiceStatus>,
    pub currency: Option<String>,
    pub exchange_rate: Option<BigDecimal>,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceListQuery {
    pub party_id: Option<PartyId>,
    pub status: Option<InvoiceStatus>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceResponse {
    pub id: InvoiceId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub invoice_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub status: String,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub exchange_rate: BigDecimal,
    pub lhdn_status: Option<String>,
    pub lhdn_uuid: Option<String>,
    pub lhdn_error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceDetailResponse {
    pub id: InvoiceId,
    pub workspace_id: WorkspaceId,
    pub party_id: PartyId,
    pub invoice_number: String,
    pub issue_date: Date,
    pub due_date: Option<Date>,
    pub status: String,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub exchange_rate: BigDecimal,
    pub lines: Vec<InvoiceLineResponse>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateInvoiceLineRequest {
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateInvoiceLineRequest {
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: Option<String>,
    pub quantity: Option<BigDecimal>,
    pub unit_price: Option<BigDecimal>,
}

#[derive(Debug, Serialize)]
pub struct InvoiceLineResponse {
    pub id: InvoiceLineId,
    pub invoice_id: InvoiceId,
    pub description: String,
    pub quantity: BigDecimal,
    pub unit_price: BigDecimal,
    pub line_total: BigDecimal,
    pub foreign_unit_price: BigDecimal,
    pub foreign_amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct PostInvoiceRequest {
    pub receivable_account_id: Option<AccountId>,
    pub revenue_account_id: Option<AccountId>,
}

#[derive(Debug, Serialize)]
pub struct PostedInvoiceResponse {
    pub invoice: InvoiceDetailResponse,
    pub journal_entry_id: JournalEntryId,
}

fn map_invoice_row(row: &sqlx::postgres::PgRow) -> Result<InvoiceResponse, sqlx::Error> {
    Ok(InvoiceResponse {
        id: InvoiceId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: PartyId(row.try_get("party_id")?),
        invoice_number: row.try_get("invoice_number")?,
        issue_date: row.try_get("issue_date")?,
        due_date: row.try_get("due_date")?,
        status: row.try_get("status")?,
        total_amount: row.try_get("total_amount")?,
        currency: row.try_get("currency")?,
        exchange_rate: row.try_get("exchange_rate")?,
        lhdn_status: row.try_get("lhdn_status")?,
        lhdn_uuid: row.try_get("lhdn_uuid")?,
        lhdn_error: row.try_get("lhdn_error")?,
    })
}

fn map_invoice_line_row(row: &sqlx::postgres::PgRow) -> Result<InvoiceLineResponse, sqlx::Error> {
    Ok(InvoiceLineResponse {
        id: InvoiceLineId(row.try_get("id")?),
        invoice_id: InvoiceId(row.try_get("invoice_id")?),
        description: row.try_get("description")?,
        quantity: row.try_get("quantity")?,
        unit_price: row.try_get("unit_price")?,
        line_total: row.try_get("line_total")?,
        foreign_unit_price: row.try_get("foreign_unit_price")?,
        foreign_amount: row.try_get("foreign_amount")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_invoices).post(create_invoice))
        .route(
            "/{id}",
            get(get_invoice)
                .patch(update_invoice)
                .delete(delete_invoice),
        )
        .route(
            "/{id}/lines",
            get(list_invoice_lines).post(create_invoice_line),
        )
        .route(
            "/{id}/lines/{line_id}",
            patch(update_invoice_line).delete(delete_invoice_line),
        )
        .route("/{id}/post", post(post_invoice))
        .route(
            "/{id}/submit-lhdn",
            post(lhdn::handlers::submit_lhdn_invoice),
        )
        .route("/{id}/lhdn-status", get(lhdn::handlers::get_lhdn_status))
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

async fn fetch_invoice(
    db: &sqlx::PgPool,
    invoice_id: InvoiceId,
    workspace_id: WorkspaceId,
) -> Result<Option<InvoiceResponse>, ApiError> {
    let row = query(
        r#"
        SELECT
            i.id, i.workspace_id, i.party_id, i.invoice_number, i.issue_date,
            i.due_date, i.status, i.total_amount, i.currency, i.exchange_rate,
            s.status AS lhdn_status,
            s.lhdn_uuid,
            s.error_message AS lhdn_error
        FROM invoice i
        LEFT JOIN LATERAL (
            SELECT status, lhdn_uuid, error_message
            FROM e_invoice_submission
            WHERE workspace_id = i.workspace_id AND invoice_id = i.id
            ORDER BY submitted_at DESC NULLS LAST, created_at DESC
            LIMIT 1
        ) s ON true
        WHERE i.id = $1 AND i.workspace_id = $2
        "#,
    )
    .bind(invoice_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(map_invoice_row(&row)?)),
        None => Ok(None),
    }
}

async fn fetch_invoice_lines(
    db: &sqlx::PgPool,
    invoice_id: InvoiceId,
) -> Result<Vec<InvoiceLineResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, invoice_id, description, quantity, unit_price, line_total,
               foreign_unit_price, foreign_amount
        FROM invoice_line
        WHERE invoice_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(invoice_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_invoice_line_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

pub(crate) async fn fetch_invoice_detail(
    db: &sqlx::PgPool,
    invoice_id: InvoiceId,
    workspace_id: WorkspaceId,
) -> Result<Option<InvoiceDetailResponse>, ApiError> {
    let invoice = match fetch_invoice(db, invoice_id, workspace_id).await? {
        Some(inv) => inv,
        None => return Ok(None),
    };

    let lines = fetch_invoice_lines(db, invoice_id).await?;

    Ok(Some(InvoiceDetailResponse {
        id: invoice.id,
        workspace_id: invoice.workspace_id,
        party_id: invoice.party_id,
        invoice_number: invoice.invoice_number,
        issue_date: invoice.issue_date,
        due_date: invoice.due_date,
        status: invoice.status,
        total_amount: invoice.total_amount,
        currency: invoice.currency,
        exchange_rate: invoice.exchange_rate,
        lines,
    }))
}

pub async fn list_invoices(
    State(state): State<AppState>,
    Query(params): Query<InvoiceListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<InvoiceResponse>>), ApiError> {
    let status = params.status.as_ref().map(|s| s.as_str().to_string());

    let rows = query(
        r#"
        SELECT
            i.id, i.workspace_id, i.party_id, i.invoice_number, i.issue_date,
            i.due_date, i.status, i.total_amount, i.currency, i.exchange_rate,
            s.status AS lhdn_status,
            s.lhdn_uuid,
            s.error_message AS lhdn_error
        FROM invoice i
        LEFT JOIN LATERAL (
            SELECT status, lhdn_uuid, error_message
            FROM e_invoice_submission
            WHERE workspace_id = i.workspace_id AND invoice_id = i.id
            ORDER BY submitted_at DESC NULLS LAST, created_at DESC
            LIMIT 1
        ) s ON true
        WHERE i.workspace_id = $1
          AND ($2::uuid IS NULL OR i.party_id = $2)
          AND ($3::text IS NULL OR i.status = $3)
        ORDER BY i.issue_date DESC, i.invoice_number
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.party_id.map(|id| id.0))
    .bind(status)
    .fetch_all(&state.db)
    .await?;

    let invoices = rows
        .iter()
        .map(map_invoice_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(invoices)))
}

pub async fn create_invoice(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), ApiError> {
    payload.validate()?;

    ensure_party_in_workspace(&state.db, payload.party_id, auth_user.workspace_id).await?;

    let invoice_id = InvoiceId::new();
    let currency = payload.currency.as_deref().unwrap_or("MYR");
    let exchange_rate = normalize_rate(payload.exchange_rate.as_ref());
    validate_currency_and_rate(currency, &exchange_rate)?;

    let row = query(
        r#"
        INSERT INTO invoice (id, workspace_id, party_id, invoice_number, issue_date, due_date, status, total_amount, currency, exchange_rate)
        VALUES ($1, $2, $3, $4, $5, $6, 'draft', 0, $7, $8)
        RETURNING id, workspace_id, party_id, invoice_number, issue_date, due_date, status, total_amount, currency, exchange_rate,
                  NULL::text AS lhdn_status, NULL::text AS lhdn_uuid, NULL::text AS lhdn_error
        "#,
    )
    .bind(invoice_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.0)
    .bind(&payload.invoice_number)
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(currency)
    .bind(&exchange_rate)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_invoice_row(&row)?)))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<InvoiceDetailResponse>), ApiError> {
    let invoice = fetch_invoice_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(invoice)))
}

pub async fn update_invoice(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceDetailResponse>), ApiError> {
    payload.validate()?;

    let existing = fetch_invoice(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(inv) => inv,
        None => return Err(ApiError::NotFound),
    };

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

    query(
        r#"
        UPDATE invoice
        SET
            party_id = COALESCE($3, party_id),
            invoice_number = COALESCE($4, invoice_number),
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
    .bind(payload.invoice_number.as_deref())
    .bind(payload.issue_date)
    .bind(payload.due_date)
    .bind(status)
    .bind(payload.currency.as_deref())
    .bind(payload.exchange_rate.as_ref())
    .execute(&mut *tx)
    .await?;

    // Recompute functional (MYR) line amounts when the exchange rate changes.
    if payload.exchange_rate.is_some()
        && payload.exchange_rate.as_ref() != Some(&existing.exchange_rate)
    {
        query(
            r#"
            UPDATE invoice_line
            SET unit_price = foreign_unit_price * $2,
                line_total = foreign_amount * $2,
                updated_at = now()
            WHERE invoice_id = $1
            "#,
        )
        .bind(id.0)
        .bind(exchange_rate)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let invoice = fetch_invoice_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(invoice)))
}

pub async fn delete_invoice(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let result = query("DELETE FROM invoice WHERE id = $1 AND workspace_id = $2")
        .bind(id.0)
        .bind(auth_user.workspace_id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_invoice_lines(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<InvoiceLineResponse>>), ApiError> {
    let exists = fetch_invoice(&state.db, id, auth_user.workspace_id).await?;
    if exists.is_none() {
        return Err(ApiError::NotFound);
    }

    let lines = fetch_invoice_lines(&state.db, id).await?;
    Ok((StatusCode::OK, Json(lines)))
}

pub async fn create_invoice_line(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
    Json(payload): Json<CreateInvoiceLineRequest>,
) -> Result<(StatusCode, Json<InvoiceLineResponse>), ApiError> {
    payload.validate()?;

    let invoice = fetch_invoice(&state.db, id, auth_user.workspace_id).await?;
    let invoice = match invoice {
        Some(inv) => inv,
        None => return Err(ApiError::NotFound),
    };

    // The request unit_price is the foreign-currency unit price.
    let foreign_unit_price = &payload.unit_price;
    let foreign_amount = &payload.quantity * foreign_unit_price;
    let myr_unit_price = foreign_unit_price * &invoice.exchange_rate;
    let myr_line_total = &foreign_amount * &invoice.exchange_rate;
    let line_id = InvoiceLineId::new();

    let row = query(
        r#"
        INSERT INTO invoice_line (
            id, invoice_id, description, quantity, unit_price, line_total,
            foreign_unit_price, foreign_amount
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, invoice_id, description, quantity, unit_price, line_total,
                 foreign_unit_price, foreign_amount
        "#,
    )
    .bind(line_id.0)
    .bind(id.0)
    .bind(&payload.description)
    .bind(&payload.quantity)
    .bind(&myr_unit_price)
    .bind(&myr_line_total)
    .bind(&foreign_unit_price)
    .bind(&foreign_amount)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::CREATED, Json(map_invoice_line_row(&row)?)))
}

pub async fn update_invoice_line(
    State(state): State<AppState>,
    Path((id, line_id)): Path<(InvoiceId, InvoiceLineId)>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateInvoiceLineRequest>,
) -> Result<(StatusCode, Json<InvoiceLineResponse>), ApiError> {
    payload.validate()?;

    let invoice = fetch_invoice(&state.db, id, auth_user.workspace_id).await?;
    let invoice = match invoice {
        Some(inv) => inv,
        None => return Err(ApiError::NotFound),
    };

    let existing =
        query("SELECT quantity, foreign_unit_price FROM invoice_line WHERE id = $1 AND invoice_id = $2")
            .bind(line_id.0)
            .bind(id.0)
            .fetch_optional(&state.db)
            .await?;

    let (quantity, foreign_unit_price) = match existing {
        Some(row) => {
            let q: BigDecimal = row.try_get("quantity")?;
            let p: BigDecimal = row.try_get("foreign_unit_price")?;
            (q, p)
        }
        None => return Err(ApiError::NotFound),
    };

    let quantity = payload.quantity.as_ref().unwrap_or(&quantity);
    let foreign_unit_price = payload.unit_price.as_ref().unwrap_or(&foreign_unit_price);
    let foreign_amount = quantity * foreign_unit_price;
    let myr_unit_price = foreign_unit_price * &invoice.exchange_rate;
    let myr_line_total = &foreign_amount * &invoice.exchange_rate;

    let row = query(
        r#"
        UPDATE invoice_line
        SET
            description = COALESCE($3, description),
            quantity = COALESCE($4, quantity),
            unit_price = $5,
            line_total = $6,
            foreign_unit_price = COALESCE($7, foreign_unit_price),
            foreign_amount = $8,
            updated_at = now()
        WHERE id = $1 AND invoice_id = $2
        RETURNING id, invoice_id, description, quantity, unit_price, line_total,
                 foreign_unit_price, foreign_amount
        "#,
    )
    .bind(line_id.0)
    .bind(id.0)
    .bind(payload.description.as_deref())
    .bind(payload.quantity.as_ref())
    .bind(&myr_unit_price)
    .bind(&myr_line_total)
    .bind(payload.unit_price.as_ref())
    .bind(&foreign_amount)
    .fetch_one(&state.db)
    .await?;

    Ok((StatusCode::OK, Json(map_invoice_line_row(&row)?)))
}

pub async fn delete_invoice_line(
    State(state): State<AppState>,
    Path((id, line_id)): Path<(InvoiceId, InvoiceLineId)>,
    auth_user: AuthUser,
) -> Result<StatusCode, ApiError> {
    let invoice_exists = fetch_invoice(&state.db, id, auth_user.workspace_id).await?;
    if invoice_exists.is_none() {
        return Err(ApiError::NotFound);
    }

    let result = query("DELETE FROM invoice_line WHERE id = $1 AND invoice_id = $2")
        .bind(line_id.0)
        .bind(id.0)
        .execute(&state.db)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
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

pub async fn post_invoice(
    State(state): State<AppState>,
    Path(id): Path<InvoiceId>,
    auth_user: AuthUser,
    Json(payload): Json<PostInvoiceRequest>,
) -> Result<(StatusCode, Json<PostedInvoiceResponse>), ApiError> {
    let invoice = fetch_invoice_detail(&state.db, id, auth_user.workspace_id).await?;
    let invoice = match invoice {
        Some(inv) => inv,
        None => return Err(ApiError::NotFound),
    };

    if invoice.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft invoices can be posted".to_string(),
        ));
    }

    if invoice.lines.is_empty() {
        return Err(ApiError::BadRequest(
            "Cannot post an invoice without lines".to_string(),
        ));
    }

    let total: BigDecimal = invoice
        .lines
        .iter()
        .map(|line| &line.line_total)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    let foreign_total: BigDecimal = invoice
        .lines
        .iter()
        .map(|line| &line.foreign_amount)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    if total <= BigDecimal::zero() {
        return Err(ApiError::BadRequest(
            "Invoice total must be greater than zero".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    let receivable_account_id = match payload.receivable_account_id {
        Some(id) => {
            ensure_account_in_workspace(&mut tx, id, auth_user.workspace_id).await?;
            id
        }
        None => {
            get_or_create_default_account(
                &mut tx,
                auth_user.workspace_id,
                "1200",
                "Accounts Receivable",
                "asset",
            )
            .await?
        }
    };

    let revenue_account_id = match payload.revenue_account_id {
        Some(id) => {
            ensure_account_in_workspace(&mut tx, id, auth_user.workspace_id).await?;
            id
        }
        None => {
            get_or_create_default_account(
                &mut tx,
                auth_user.workspace_id,
                "4000",
                "Revenue",
                "revenue",
            )
            .await?
        }
    };

    query(
        r#"
        UPDATE invoice
        SET status = 'posted', total_amount = $3, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&total)
    .execute(&mut *tx)
    .await?;

    let journal_entry_id = JournalEntryId::new();
    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(invoice.issue_date)
    .bind(&invoice.invoice_number)
    .bind(format!("Invoice {}", invoice.invoice_number))
    .execute(&mut *tx)
    .await?;

    query(
        r#"
        INSERT INTO journal_line (
            id, journal_entry_id, account_id, party_id, description,
            debit, credit, foreign_debit, foreign_credit, exchange_rate
        )
        VALUES ($1, $2, $3, $4, $5, $6, 0, $7, 0, $8)
        "#,
    )
    .bind(JournalLineId::new().0)
    .bind(journal_entry_id.0)
    .bind(receivable_account_id.0)
    .bind(invoice.party_id.0)
    .bind(format!("Accounts Receivable - Invoice {}", invoice.invoice_number))
    .bind(&total)
    .bind(&foreign_total)
    .bind(&invoice.exchange_rate)
    .execute(&mut *tx)
    .await?;

    query(
        r#"
        INSERT INTO journal_line (
            id, journal_entry_id, account_id, party_id, description,
            debit, credit, foreign_debit, foreign_credit, exchange_rate
        )
        VALUES ($1, $2, $3, $4, $5, 0, $6, 0, $7, $8)
        "#,
    )
    .bind(JournalLineId::new().0)
    .bind(journal_entry_id.0)
    .bind(revenue_account_id.0)
    .bind(invoice.party_id.0)
    .bind(format!("Revenue - Invoice {}", invoice.invoice_number))
    .bind(&total)
    .bind(&foreign_total)
    .bind(&invoice.exchange_rate)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let invoice = fetch_invoice_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((
        StatusCode::OK,
        Json(PostedInvoiceResponse {
            invoice,
            journal_entry_id,
        }),
    ))
}
