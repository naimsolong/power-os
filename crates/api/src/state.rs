// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use sqlx::PgPool;

use crate::lhdn::LhdnClient;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub lhdn: LhdnClient,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            lhdn: LhdnClient::new(),
        }
    }
}
