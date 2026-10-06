// SPDX-License-Identifier: MIT
// Copyright (c) 2026 Power OS Contributors

use sqlx::PgPool;
use std::env;

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPool::connect(&database_url).await?;
    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(())
}
