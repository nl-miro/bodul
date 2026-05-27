mod common;

use dotenvy::dotenv;
use retailer_sourcing::io::build_pool;
use std::env::var;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = build_pool(&database_url)?;
    common::truncate_all_tables(&pool)?;
    println!("Truncated all tables in {database_url}");
    Ok(())
}
