#![allow(dead_code)]

use dotenvy::dotenv;
use retailer_sourcing::io::{DbPool, build_pool};
use retailer_sourcing::run_migrations;
use std::env::var;
use std::fmt::Display;
use std::io::Error;

pub fn database_pool() -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    database_pool_from_url(&database_url)
}

pub fn database_pool_from_url(
    database_url: &str,
) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let pool = build_pool(database_url)?;
    run_migrations(&pool)?;
    Ok(pool)
}

pub fn truncate_all_tables(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use diesel::prelude::*;
    use diesel::sql_query;

    let mut conn = pool.get()?;
    sql_query(
        "TRUNCATE TABLE command_entries, event_entries, inbox_entries, outbox_entries \
         RESTART IDENTITY CASCADE",
    )
    .execute(&mut conn)?;
    Ok(())
}
pub fn batch_error<T>(kind: &str, errors: Vec<T>) -> Error
where
    T: Display,
{
    for error in &errors {
        eprintln!("{kind}: {error}");
    }

    Error::other(format!("failed to process {kind}"))
}
