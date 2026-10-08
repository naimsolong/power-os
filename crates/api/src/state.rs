// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use sqlx::PgPool;

use crate::ai::{AiClient, AiConfig};
use crate::lhdn::LhdnClient;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub lhdn: LhdnClient,
    pub ai: AiClient,
}

impl AppState {
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            lhdn: LhdnClient::new(),
            ai: AiClient::new(AiConfig::from_env()),
        }
    }
}
