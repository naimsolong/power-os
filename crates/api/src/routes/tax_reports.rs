// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use bigdecimal::{BigDecimal, Zero};
use power_os_domain::TaxCodeId;
use serde::{Deserialize, Serialize};
use sqlx::{query, Row};
use time::Date;

use crate::auth::AuthUser;
use crate::routes::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct Sst02ReportQuery {
    pub from: Date,
    pub to: Date,
}

#[derive(Debug, Serialize)]
pub struct Sst02Line {
    pub tax_code_id: TaxCodeId,
    pub code: String,
    pub description: String,
    pub rate: BigDecimal,
    pub taxable_amount: BigDecimal,
    pub tax_amount: BigDecimal,
}

#[derive(Debug, Serialize)]
pub struct Sst02ReportResponse {
    pub from: Date,
    pub to: Date,
    pub total_taxable_amount: BigDecimal,
    pub total_tax_amount: BigDecimal,
    pub lines: Vec<Sst02Line>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/sst-02", get(sst_02_report))
}

pub async fn sst_02_report(
    State(state): State<AppState>,
    auth_user: AuthUser,
    Query(params): Query<Sst02ReportQuery>,
) -> Result<(StatusCode, Json<Sst02ReportResponse>), ApiError> {
    if params.from > params.to {
        return Err(ApiError::BadRequest(
            "From date must be before or equal to to date".to_string(),
        ));
    }

    let rows = query(
        r#"
        SELECT
            tc.id AS tax_code_id,
            tc.code,
            tc.description,
            tc.rate,
            COALESCE(SUM(il.line_total), 0) AS taxable_amount,
            COALESCE(SUM(il.tax_amount), 0) AS tax_amount
        FROM invoice_line il
        JOIN invoice i ON i.id = il.invoice_id
        JOIN tax_code tc ON tc.id = il.tax_code_id
        WHERE i.workspace_id = $1
          AND i.status = 'posted'
          AND i.issue_date BETWEEN $2 AND $3
          AND il.tax_code_id IS NOT NULL
        GROUP BY tc.id, tc.code, tc.description, tc.rate
        ORDER BY tc.code
        "#,
    )
    .bind(auth_user.workspace_id.0)
    .bind(params.from)
    .bind(params.to)
    .fetch_all(&state.db)
    .await?;

    let mut lines = Vec::with_capacity(rows.len());
    let mut total_taxable_amount = BigDecimal::zero();
    let mut total_tax_amount = BigDecimal::zero();

    for row in rows {
        let taxable_amount: BigDecimal = row.try_get("taxable_amount")?;
        let tax_amount: BigDecimal = row.try_get("tax_amount")?;
        total_taxable_amount += &taxable_amount;
        total_tax_amount += &tax_amount;

        lines.push(Sst02Line {
            tax_code_id: TaxCodeId(row.try_get("tax_code_id")?),
            code: row.try_get("code")?,
            description: row.try_get("description")?,
            rate: row.try_get("rate")?,
            taxable_amount,
            tax_amount,
        });
    }

    Ok((
        StatusCode::OK,
        Json(Sst02ReportResponse {
            from: params.from,
            to: params.to,
            total_taxable_amount,
            total_tax_amount,
            lines,
        }),
    ))
}
