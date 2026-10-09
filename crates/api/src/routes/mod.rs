// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, routing::post, Router};
use tower_http::services::{ServeDir, ServeFile};

use crate::ai;
use crate::auth;
use crate::lhdn;
use crate::state::AppState;

pub(crate) mod accounts;
mod deal_stages;
mod deals;
pub(crate) mod employees;
pub(crate) mod error;
pub(crate) mod fiscal_periods;
mod health;
pub(crate) mod invoices;
mod journal_entries;
mod parties;
pub(crate) mod tax_codes;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .nest("/api/auth", auth::router())
        .nest("/api/parties", parties::router())
        .nest("/api/deal-stages", deal_stages::router())
        .nest("/api/deals", deals::router())
        .nest("/api/accounts", accounts::router())
        .nest("/api/employees", employees::router())
        .nest("/api/invoices", invoices::router())
        .nest("/api/journal-entries", journal_entries::router())
        .nest("/api/tax-codes", tax_codes::router())
        .nest("/api/fiscal-years", fiscal_periods::router())
        .nest("/api/accounting-periods", fiscal_periods::accounting_periods_router())
        .route("/api/ai/chat", post(ai::handlers::chat))
        .route(
            "/api/workspace/lhdn-settings",
            axum::routing::get(lhdn::handlers::get_lhdn_settings)
                .patch(lhdn::handlers::update_lhdn_settings),
        )
        .fallback_service(static_service())
        .with_state(state)
}

fn static_service() -> ServeDir<ServeFile> {
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "frontend/dist".to_string());
    let index_path = format!("{}/index.html", static_dir);
    ServeDir::new(&static_dir).fallback(ServeFile::new(index_path))
}
