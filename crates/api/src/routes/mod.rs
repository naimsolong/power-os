// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use axum::{routing::get, Router};

mod health;

pub fn router() -> Router {
    Router::new().route("/health", get(health::health))
}
