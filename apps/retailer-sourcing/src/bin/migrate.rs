use dotenvy::dotenv;
use retailer_sourcing::{io::build_pool, run_migrations};
use std::env::var;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();
    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = build_pool(&database_url)?;
    run_migrations(&pool)?;
    println!("Migrations applied successfully.");
    Ok(())
}
