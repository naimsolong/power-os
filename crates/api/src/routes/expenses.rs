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
    AccountId, ExpenseId, ExpenseLineId, JournalEntryId, JournalLineId, PartyId, WorkspaceId,
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
pub enum ExpenseStatus {
    Draft,
    Posted,
    Cancelled,
}

impl ExpenseStatus {
    fn as_str(&self) -> &'static str {
        match self {
            ExpenseStatus::Draft => "draft",
            ExpenseStatus::Posted => "posted",
            ExpenseStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateExpenseLineRequest {
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    pub account_id: AccountId,
    pub amount: BigDecimal,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateExpenseRequest {
    pub party_id: Option<PartyId>,
    pub expense_date: Date,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: String,
    pub reference: Option<String>,
    pub payment_method: Option<String>,
    pub paid_from_account_id: Option<AccountId>,
    pub currency: Option<String>,
    pub lines: Vec<CreateExpenseLineRequest>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateExpenseRequest {
    pub party_id: Option<PartyId>,
    pub expense_date: Option<Date>,
    #[validate(length(min = 1, message = "Description is required"))]
    pub description: Option<String>,
    pub reference: Option<String>,
    pub payment_method: Option<String>,
    pub paid_from_account_id: Option<AccountId>,
    pub status: Option<ExpenseStatus>,
    pub currency: Option<String>,
    pub lines: Option<Vec<CreateExpenseLineRequest>>,
}

#[derive(Debug, Deserialize)]
pub struct ExpenseListQuery {
    pub party_id: Option<PartyId>,
    pub status: Option<ExpenseStatus>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseResponse {
    pub id: ExpenseId,
    pub workspace_id: WorkspaceId,
    pub party_id: Option<PartyId>,
    pub expense_date: Date,
    pub description: String,
    pub reference: Option<String>,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub payment_method: Option<String>,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ExpenseDetailResponse {
    pub id: ExpenseId,
    pub workspace_id: WorkspaceId,
    pub party_id: Option<PartyId>,
    pub expense_date: Date,
    pub description: String,
    pub reference: Option<String>,
    pub total_amount: BigDecimal,
    pub currency: String,
    pub payment_method: Option<String>,
    pub status: String,
    pub paid_from_account_id: Option<AccountId>,
    pub journal_entry_id: Option<JournalEntryId>,
    pub lines: Vec<ExpenseLineResponse>,
}

#[derive(Debug, Serialize)]
pub struct ExpenseLineResponse {
    pub id: ExpenseLineId,
    pub expense_id: ExpenseId,
    pub description: String,
    pub account_id: AccountId,
    pub amount: BigDecimal,
}

#[derive(Debug, Deserialize)]
pub struct PostExpenseRequest {
    pub payable_account_id: Option<AccountId>,
}

#[derive(Debug, Serialize)]
pub struct PostedExpenseResponse {
    pub expense: ExpenseDetailResponse,
    pub journal_entry_id: JournalEntryId,
}

fn map_expense_row(row: &sqlx::postgres::PgRow) -> Result<ExpenseResponse, sqlx::Error> {
    Ok(ExpenseResponse {
        id: ExpenseId(row.try_get("id")?),
        workspace_id: WorkspaceId(row.try_get("workspace_id")?),
        party_id: row.try_get("party_id")?,
        expense_date: row.try_get("expense_date")?,
        description: row.try_get("description")?,
        reference: row.try_get("reference")?,
        total_amount: row.try_get("total_amount")?,
        currency: row.try_get("currency")?,
        payment_method: row.try_get("payment_method")?,
        status: row.try_get("status")?,
    })
}

fn map_expense_line_row(
    row: &sqlx::postgres::PgRow,
) -> Result<ExpenseLineResponse, sqlx::Error> {
    Ok(ExpenseLineResponse {
        id: ExpenseLineId(row.try_get("id")?),
        expense_id: ExpenseId(row.try_get("expense_id")?),
        description: row.try_get("description")?,
        account_id: AccountId(row.try_get("account_id")?),
        amount: row.try_get("amount")?,
    })
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_expenses).post(create_expense))
        .route("/{id}", get(get_expense).patch(update_expense))
        .route("/{id}/post", post(post_expense))
        .route("/{id}/cancel", post(cancel_expense))
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

async fn fetch_expense(
    db: &sqlx::PgPool,
    expense_id: ExpenseId,
    workspace_id: WorkspaceId,
) -> Result<Option<ExpenseResponse>, ApiError> {
    let row = query(
        r#"
        SELECT id, workspace_id, party_id, expense_date, description, reference,
               total_amount, currency, payment_method, status
        FROM expense
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(expense_id.0)
    .bind(workspace_id.0)
    .fetch_optional(db)
    .await?;

    match row {
        Some(row) => Ok(Some(map_expense_row(&row)?)),
        None => Ok(None),
    }
}

async fn fetch_expense_lines(
    db: &sqlx::PgPool,
    expense_id: ExpenseId,
) -> Result<Vec<ExpenseLineResponse>, ApiError> {
    let rows = query(
        r#"
        SELECT id, expense_id, description, account_id, amount
        FROM expense_line
        WHERE expense_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(expense_id.0)
    .fetch_all(db)
    .await?;

    rows.iter()
        .map(map_expense_line_row)
        .collect::<Result<Vec<_>, _>>()
        .map_err(ApiError::from)
}

async fn fetch_expense_detail(
    db: &sqlx::PgPool,
    expense_id: ExpenseId,
    workspace_id: WorkspaceId,
) -> Result<Option<ExpenseDetailResponse>, ApiError> {
    let expense = match fetch_expense(db, expense_id, workspace_id).await? {
        Some(e) => e,
        None => return Ok(None),
    };

    let detail_row = query(
        r#"
        SELECT paid_from_account_id, journal_entry_id
        FROM expense
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(expense_id.0)
    .bind(workspace_id.0)
    .fetch_one(db)
    .await?;

    let paid_from_account_id: Option<AccountId> = detail_row
        .try_get::<Option<uuid::Uuid>, _>("paid_from_account_id")?
        .map(AccountId::from);
    let journal_entry_id: Option<JournalEntryId> = detail_row
        .try_get::<Option<uuid::Uuid>, _>("journal_entry_id")?
        .map(JournalEntryId::from);

    let lines = fetch_expense_lines(db, expense_id).await?;

    Ok(Some(ExpenseDetailResponse {
        id: expense.id,
        workspace_id: expense.workspace_id,
        party_id: expense.party_id,
        expense_date: expense.expense_date,
        description: expense.description,
        reference: expense.reference,
        total_amount: expense.total_amount,
        currency: expense.currency,
        payment_method: expense.payment_method,
        status: expense.status,
        paid_from_account_id,
        journal_entry_id,
        lines,
    }))
}

async fn insert_expense_lines(
    tx: &mut sqlx::PgConnection,
    expense_id: ExpenseId,
    lines: &[CreateExpenseLineRequest],
) -> Result<BigDecimal, ApiError> {
    let mut total = BigDecimal::zero();

    for line in lines {
        let line_id = ExpenseLineId::new();

        query(
            r#"
            INSERT INTO expense_line (id, expense_id, description, account_id, amount)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(line_id.0)
        .bind(expense_id.0)
        .bind(&line.description)
        .bind(line.account_id.0)
        .bind(&line.amount)
        .execute(&mut *tx)
        .await?;

        total = total + &line.amount;
    }

    Ok(total)
}

pub async fn list_expenses(
    State(state): State<AppState>,
    Query(params): Query<ExpenseListQuery>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<Vec<ExpenseResponse>>), ApiError> {
    let status = params.status.as_ref().map(|s| s.as_str().to_string());
    let search = params.search.as_deref().unwrap_or("").trim();

    let rows = query(
        r#"
        SELECT id, workspace_id, party_id, expense_date, description, reference,
               total_amount, currency, payment_method, status
        FROM expense
        WHERE workspace_id = $1
          AND ($2::uuid IS NULL OR party_id = $2)
          AND ($3::text IS NULL OR status = $3)
          AND (
              $4::text = '' OR
              description ILIKE '%' || $4 || '%' OR
              reference ILIKE '%' || $4 || '%'
          )
        ORDER BY expense_date DESC, description
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.party_id.map(|id| id.0))
    .bind(status)
    .bind(search)
    .fetch_all(&state.db)
    .await?;

    let expenses = rows
        .iter()
        .map(map_expense_row)
        .collect::<Result<Vec<_>, _>>()?;

    Ok((StatusCode::OK, Json(expenses)))
}

pub async fn create_expense(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Json(payload): Json<CreateExpenseRequest>,
) -> Result<(StatusCode, Json<ExpenseDetailResponse>), ApiError> {
    payload.validate()?;

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    let expense_id = ExpenseId::new();
    let currency = payload.currency.as_deref().unwrap_or("MYR");

    let mut tx = state.db.begin().await?;

    for line in &payload.lines {
        ensure_account_in_workspace(&mut tx, line.account_id, auth_user.workspace_id).await?;
    }

    if let Some(account_id) = payload.paid_from_account_id {
        ensure_account_in_workspace(&mut tx, account_id, auth_user.workspace_id).await?;
    }

    query(
        r#"
        INSERT INTO expense (id, workspace_id, party_id, expense_date, description, reference,
                             total_amount, currency, payment_method, status, paid_from_account_id)
        VALUES ($1, $2, $3, $4, $5, $6, 0, $7, $8, 'draft', $9)
        "#,
    )
    .bind(expense_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|p| p.0))
    .bind(payload.expense_date)
    .bind(&payload.description)
    .bind(payload.reference.as_deref())
    .bind(currency)
    .bind(payload.payment_method.as_deref())
    .bind(payload.paid_from_account_id.map(|a| a.0))
    .execute(&mut *tx)
    .await?;

    let total = insert_expense_lines(&mut tx, expense_id, &payload.lines).await?;

    query(
        r#"
        UPDATE expense
        SET total_amount = $3, updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(expense_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(&total)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let expense = fetch_expense_detail(&state.db, expense_id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::Internal)?;

    Ok((StatusCode::CREATED, Json(expense)))
}

pub async fn get_expense(
    State(state): State<AppState>,
    Path(id): Path<ExpenseId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<ExpenseDetailResponse>), ApiError> {
    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(expense)))
}

pub async fn update_expense(
    State(state): State<AppState>,
    Path(id): Path<ExpenseId>,
    auth_user: AuthUser,
    Json(payload): Json<UpdateExpenseRequest>,
) -> Result<(StatusCode, Json<ExpenseDetailResponse>), ApiError> {
    payload.validate()?;

    let existing = fetch_expense(&state.db, id, auth_user.workspace_id).await?;
    let existing = match existing {
        Some(e) => e,
        None => return Err(ApiError::NotFound),
    };

    if existing.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft expenses can be updated".to_string(),
        ));
    }

    if let Some(party_id) = payload.party_id {
        ensure_party_in_workspace(&state.db, party_id, auth_user.workspace_id).await?;
    }

    let status = payload.status.as_ref().map(|s| s.as_str());

    let mut tx = state.db.begin().await?;

    if let Some(lines) = &payload.lines {
        for line in lines {
            ensure_account_in_workspace(&mut tx, line.account_id, auth_user.workspace_id).await?;
        }
    }

    if let Some(account_id) = payload.paid_from_account_id {
        ensure_account_in_workspace(&mut tx, account_id, auth_user.workspace_id).await?;
    }

    query(
        r#"
        UPDATE expense
        SET
            party_id = COALESCE($3, party_id),
            expense_date = COALESCE($4, expense_date),
            description = COALESCE($5, description),
            reference = COALESCE($6, reference),
            payment_method = COALESCE($7, payment_method),
            paid_from_account_id = COALESCE($8, paid_from_account_id),
            status = COALESCE($9, status),
            currency = COALESCE($10, currency),
            updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .bind(payload.party_id.map(|p| p.0))
    .bind(payload.expense_date)
    .bind(payload.description.as_deref())
    .bind(payload.reference.as_deref())
    .bind(payload.payment_method.as_deref())
    .bind(payload.paid_from_account_id.map(|a| a.0))
    .bind(status)
    .bind(payload.currency.as_deref())
    .execute(&mut *tx)
    .await?;

    let total = if let Some(lines) = payload.lines {
        query("DELETE FROM expense_line WHERE expense_id = $1")
            .bind(id.0)
            .execute(&mut *tx)
            .await?;
        insert_expense_lines(&mut tx, id, &lines).await?
    } else {
        existing.total_amount
    };

    query(
        r#"
        UPDATE expense
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

    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(expense)))
}

pub async fn post_expense(
    State(state): State<AppState>,
    Path(id): Path<ExpenseId>,
    auth_user: AuthUser,
    Json(payload): Json<PostExpenseRequest>,
) -> Result<(StatusCode, Json<PostedExpenseResponse>), ApiError> {
    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id).await?;
    let expense = match expense {
        Some(e) => e,
        None => return Err(ApiError::NotFound),
    };

    if expense.status != "draft" {
        return Err(ApiError::BadRequest(
            "Only draft expenses can be posted".to_string(),
        ));
    }

    if expense.lines.is_empty() {
        return Err(ApiError::BadRequest(
            "Cannot post an expense without lines".to_string(),
        ));
    }

    let total: BigDecimal = expense
        .lines
        .iter()
        .map(|line| &line.amount)
        .fold(BigDecimal::zero(), |acc, x| acc + x);

    if total <= BigDecimal::zero() {
        return Err(ApiError::BadRequest(
            "Expense total must be greater than zero".to_string(),
        ));
    }

    let mut tx = state.db.begin().await?;

    let credit_account_id = match expense.paid_from_account_id {
        Some(id) => {
            ensure_account_in_workspace(&mut tx, id, auth_user.workspace_id).await?;
            id
        }
        None => match payload.payable_account_id {
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
        },
    };

    let journal_entry_id = JournalEntryId::new();
    let reference = expense
        .reference
        .as_deref()
        .unwrap_or(&expense.description);

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(journal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(expense.expense_date)
    .bind(reference)
    .bind(format!("Expense: {}", expense.description))
    .execute(&mut *tx)
    .await?;

    for line in &expense.lines {
        query(
            r#"
            INSERT INTO journal_line (id, journal_entry_id, account_id, party_id, description, debit, credit)
            VALUES ($1, $2, $3, $4, $5, $6, 0)
            "#,
        )
        .bind(JournalLineId::new().0)
        .bind(journal_entry_id.0)
        .bind(line.account_id.0)
        .bind(expense.party_id.map(|p| p.0))
        .bind(format!("{} - Expense {}", line.description, reference))
        .bind(&line.amount)
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
    .bind(credit_account_id.0)
    .bind(expense.party_id.map(|p| p.0))
    .bind(format!("Credit - Expense {}", reference))
    .bind(&total)
    .execute(&mut *tx)
    .await?;

    query(
        r#"
        UPDATE expense
        SET status = 'posted', total_amount = $3, journal_entry_id = $4, updated_at = now()
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

    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((
        StatusCode::OK,
        Json(PostedExpenseResponse {
            expense,
            journal_entry_id,
        }),
    ))
}

pub async fn cancel_expense(
    State(state): State<AppState>,
    Path(id): Path<ExpenseId>,
    auth_user: AuthUser,
) -> Result<(StatusCode, Json<ExpenseDetailResponse>), ApiError> {
    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id).await?;
    let expense = match expense {
        Some(e) => e,
        None => return Err(ApiError::NotFound),
    };

    if expense.status != "posted" {
        return Err(ApiError::BadRequest(
            "Only posted expenses can be cancelled".to_string(),
        ));
    }

    let original_journal_entry_id = match expense.journal_entry_id {
        Some(id) => id,
        None => return Err(ApiError::BadRequest(
            "Expense has no journal entry".to_string(),
        )),
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
    let reference = expense
        .reference
        .as_deref()
        .unwrap_or(&expense.description);

    query(
        r#"
        INSERT INTO journal_entry (id, workspace_id, entry_date, reference, description, status)
        VALUES ($1, $2, $3, $4, $5, 'posted')
        "#,
    )
    .bind(reversal_entry_id.0)
    .bind(auth_user.workspace_id.0)
    .bind(today)
    .bind(format!("CNCL-{}", reference))
    .bind(format!("Cancellation of expense {}", reference))
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
        UPDATE expense
        SET status = 'cancelled', updated_at = now()
        WHERE id = $1 AND workspace_id = $2
        "#,
    )
    .bind(id.0)
    .bind(auth_user.workspace_id.0)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let expense = fetch_expense_detail(&state.db, id, auth_user.workspace_id)
        .await?
        .ok_or(ApiError::NotFound)?;

    Ok((StatusCode::OK, Json(expense)))
}
