mod common;

use dotenvy::dotenv;
use retailer_sourcing::io::build_pool;
use std::env::var;

const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://postgres:postgres@localhost:26001/retailer_sourcing_test";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url =
        var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
    let pool = build_pool(&database_url)?;
    common::truncate_all_tables(&pool)?;
    println!("Truncated all tables in {database_url}");
    Ok(())
}
