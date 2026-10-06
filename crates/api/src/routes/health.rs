// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

pub async fn health() -> impl IntoResponse {
    let body = HealthResponse { status: "ok" };
    (StatusCode::OK, Json(body))
}
