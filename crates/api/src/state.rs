// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}
