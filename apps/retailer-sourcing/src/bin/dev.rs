mod common;

use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use kernel::io::ReservableCommandSpec;
use kernel::io::ReservableEventSpec;
use kernel::{KernelConfig, boot};
use retailer_sourcing::io::{DbPool, setup_full_kernel_with_pool};
use std::env::var;

#[derive(Parser)]
#[command(about = "retailer-sourcing dev/main DB ops")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    ProcessCommands,
    ProcessEvents,
    Truncate,
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cli = Cli::parse();
    dotenv().ok();

    let url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db_pool = common::database_pool_from_url(&url)?;

    match cli.cmd {
        Cmd::ProcessCommands => run_commands(db_pool),
        Cmd::ProcessEvents => run_events(db_pool),
        Cmd::Truncate => truncate_all_tables(&db_pool),
    }
}

//
//

//
//
//
//
//
//
//

const MAX_COMMANDS_PER_RUN: usize = 10;

pub fn run_commands(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async move {
        let kernel = setup_full_kernel_with_pool(pool);
        let consumer = kernel.handle.command_consumer();

        let result = tokio::task::spawn_blocking(move || {
            consumer.consume(&ReservableCommandSpec::new(MAX_COMMANDS_PER_RUN))
        })
        .await?;

        match result {
            Ok(processed_count) => println!("Processed {processed_count} commands."),
            Err(errors) => {
                println!(
                    "Command processing finished with {} error(s); see stderr for details.",
                    errors.len()
                );
            }
        }

        kernel.shutdown().await;
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(())
    })
}

const MAX_EVENTS_PER_RUN: usize = 10;

pub fn run_events(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kernel = boot(KernelConfig::default()).start_persistent(pool, 0)?;

    let result = kernel
        .event_consumer()
        .consume(&ReservableEventSpec::new(MAX_EVENTS_PER_RUN));

    match result {
        Ok(processed_count) => println!("Processed {processed_count} events."),
        Err(errors) => {
            println!(
                "Event processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
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
