// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use power_os_domain::WorkspaceId;
use sqlx::{query, Row};
use time::Date;

use crate::routes::error::ApiError;

/// Checks whether the accounting period containing `date` for the workspace is
/// closed. Returns `ApiError::Conflict` when the period is closed.
pub(crate) async fn ensure_period_open(
    db: &sqlx::PgPool,
    workspace_id: WorkspaceId,
    date: Date,
) -> Result<(), ApiError> {
    let row = query(
        r#"
        SELECT is_closed
        FROM accounting_period
        WHERE workspace_id = $1
          AND start_date <= $2
          AND end_date >= $2
        "#,
    )
    .bind(workspace_id.0)
    .bind(date)
    .fetch_optional(db)
    .await?;

    if let Some(row) = row {
        let is_closed: bool = row.try_get("is_closed")?;
        if is_closed {
            return Err(ApiError::Conflict(format!(
                "Accounting period for {} is closed.",
                date
            )));
        }
    }

    Ok(())
}
