// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health(_state: State<AppState>) -> impl IntoResponse {
    let body = HealthResponse { status: "ok" };
    (StatusCode::OK, Json(body))
}
