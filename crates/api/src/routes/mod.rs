// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, routing::post, Router};

use crate::ai;
use crate::auth;
use crate::lhdn;
use crate::state::AppState;

mod deal_stages;
mod deals;
pub(crate) mod employees;
pub(crate) mod error;
mod health;
pub(crate) mod invoices;
mod journal_entries;
mod parties;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .nest("/api/auth", auth::router())
        .nest("/api/parties", parties::router())
        .nest("/api/deal-stages", deal_stages::router())
        .nest("/api/deals", deals::router())
        .nest("/api/employees", employees::router())
        .nest("/api/invoices", invoices::router())
        .nest("/api/journal-entries", journal_entries::router())
        .route("/api/ai/chat", post(ai::handlers::chat))
        .route(
            "/api/workspace/lhdn-settings",
            axum::routing::get(lhdn::handlers::get_lhdn_settings)
                .patch(lhdn::handlers::update_lhdn_settings),
        )
        .with_state(state)
}
