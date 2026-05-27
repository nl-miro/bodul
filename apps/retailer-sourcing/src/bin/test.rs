mod common;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use retailer_sourcing::io::build_pool;
use std::env::var;

const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://postgres:postgres@localhost:26001/retailer_sourcing_test";

#[derive(Parser)]
#[command(about = "retailer-sourcing test DB ops")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    ProcessInbox,
    ProcessCommands,
    ProcessEvents,
    ProcessOutbox,
    Truncate,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    dotenv().ok();
    let url = var("TEST_DATABASE_URL").unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());

    match cli.cmd {
        Cmd::ProcessInbox => common::run_inbox(common::database_pool_from_url(&url)?),
        Cmd::ProcessCommands => common::run_commands(common::database_pool_from_url(&url)?),
        Cmd::ProcessEvents => common::run_events(common::database_pool_from_url(&url)?),
        Cmd::ProcessOutbox => common::run_outbox(common::database_pool_from_url(&url)?),
        Cmd::Truncate => {
            let pool = build_pool(&url)?;
            common::truncate_all_tables(&pool)?;
            println!("Truncated all tables in {url}");
            Ok(())
        }
    }
}
