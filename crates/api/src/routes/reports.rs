// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use bigdecimal::BigDecimal;
use power_os_domain::AccountId;
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    pub as_of: Date,
}

#[derive(Debug, Deserialize)]
pub struct ProfitLossQuery {
    pub from: Date,
    pub to: Date,
}

#[derive(Debug, Deserialize)]
pub struct BalanceSheetQuery {
    pub as_of: Date,
}

#[derive(Debug, Serialize)]
pub struct TrialBalanceLine {
    pub account_id: AccountId,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub debit: BigDecimal,
    pub credit: BigDecimal,
    pub balance: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct TrialBalanceResponse {
    pub as_of: Date,
    pub lines: Vec<TrialBalanceLine>,
}

#[derive(Debug, Serialize)]
pub struct ProfitLossResponse {
    pub from: Date,
    pub to: Date,
    pub revenue: BigDecimal,
    pub expenses: BigDecimal,
    pub net_profit: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct BalanceSheetResponse {
    pub as_of: Date,
    pub assets: BigDecimal,
    pub liabilities: BigDecimal,
    pub equity: BigDecimal,
    pub retained_earnings: BigDecimal,
    pub check: BigDecimal,
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/trial-balance", get(trial_balance))
        .route("/profit-loss", get(profit_loss))
        .route("/balance-sheet", get(balance_sheet))
}

pub async fn trial_balance(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<TrialBalanceQuery>,
) -> Result<(StatusCode, Json<TrialBalanceResponse>), ApiError> {
    let rows = query(
        r#"
        SELECT
            a.id AS account_id,
            a.code AS account_code,
            a.name AS account_name,
            a.account_type,
            COALESCE(SUM(jl.debit), 0) AS debit,
            COALESCE(SUM(jl.credit), 0) AS credit
        FROM account a
        LEFT JOIN journal_line jl ON jl.account_id = a.id
        LEFT JOIN journal_entry je ON je.id = jl.journal_entry_id
            AND je.status = 'posted'
            AND je.entry_date <= $2
        WHERE a.workspace_id = $1
        GROUP BY a.id, a.code, a.name, a.account_type
        ORDER BY a.code
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.as_of)
    .fetch_all(&state.db)
    .await?;

    let mut lines = Vec::with_capacity(rows.len());
    for row in rows {
        let debit: BigDecimal = row.try_get("debit")?;
        let credit: BigDecimal = row.try_get("credit")?;
        let balance = &debit - &credit;

        lines.push(TrialBalanceLine {
            account_id: AccountId(row.try_get("account_id")?),
            account_code: row.try_get("account_code")?,
            account_name: row.try_get("account_name")?,
            account_type: row.try_get("account_type")?,
            debit,
            credit,
            balance,
        });
    }

    Ok((
        StatusCode::OK,
        Json(TrialBalanceResponse {
            as_of: params.as_of,
            lines,
        }),
    ))
}

pub async fn profit_loss(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<ProfitLossQuery>,
) -> Result<(StatusCode, Json<ProfitLossResponse>), ApiError> {
    if params.from > params.to {
        return Err(ApiError::BadRequest(
            "From date must be before or equal to to date".to_string(),
        ));
    }

    let row = query(
        r#"
        SELECT
            COALESCE(SUM(
                CASE WHEN a.account_type = 'revenue' THEN jl.credit - jl.debit ELSE 0 END
            ), 0) AS revenue,
            COALESCE(SUM(
                CASE WHEN a.account_type = 'expense' THEN jl.debit - jl.credit ELSE 0 END
            ), 0) AS expenses
        FROM journal_line jl
        JOIN account a ON a.id = jl.account_id
        JOIN journal_entry je ON je.id = jl.journal_entry_id
        WHERE a.workspace_id = $1
          AND je.status = 'posted'
          AND je.entry_date BETWEEN $2 AND $3
          AND a.account_type IN ('revenue', 'expense')
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.from)
    .bind(params.to)
    .fetch_one(&state.db)
    .await?;

    let revenue: BigDecimal = row.try_get("revenue")?;
    let expenses: BigDecimal = row.try_get("expenses")?;
    let net_profit = &revenue - &expenses;

    Ok((
        StatusCode::OK,
        Json(ProfitLossResponse {
            from: params.from,
            to: params.to,
            revenue,
            expenses,
            net_profit,
        }),
    ))
}

pub async fn balance_sheet(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<BalanceSheetQuery>,
) -> Result<(StatusCode, Json<BalanceSheetResponse>), ApiError> {
    let row = query(
        r#"
        SELECT
            COALESCE(SUM(
                CASE WHEN a.account_type = 'asset' THEN jl.debit - jl.credit ELSE 0 END
            ), 0) AS assets,
            COALESCE(SUM(
                CASE WHEN a.account_type = 'liability' THEN jl.credit - jl.debit ELSE 0 END
            ), 0) AS liabilities,
            COALESCE(SUM(
                CASE WHEN a.account_type = 'equity' THEN jl.credit - jl.debit ELSE 0 END
            ), 0) AS equity,
            COALESCE(SUM(
                CASE WHEN a.account_type IN ('revenue', 'expense') THEN jl.credit - jl.debit ELSE 0 END
            ), 0) AS retained_earnings
        FROM journal_line jl
        JOIN account a ON a.id = jl.account_id
        JOIN journal_entry je ON je.id = jl.journal_entry_id
        WHERE a.workspace_id = $1
          AND je.status = 'posted'
          AND je.entry_date <= $2
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.as_of)
    .fetch_one(&state.db)
    .await?;

    let assets: BigDecimal = row.try_get("assets")?;
    let liabilities: BigDecimal = row.try_get("liabilities")?;
    let equity_accounts: BigDecimal = row.try_get("equity")?;
    let retained_earnings: BigDecimal = row.try_get("retained_earnings")?;
    let equity = &equity_accounts + &retained_earnings;
    let check = &assets - &liabilities - &equity;

    Ok((
        StatusCode::OK,
        Json(BalanceSheetResponse {
            as_of: params.as_of,
            assets,
            liabilities,
            equity,
            retained_earnings,
            check,
        }),
    ))
}

