mod common;

use diesel::QueryableByName;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer};
use kernel::io::ReservableCommandSpec;
use kernel::{KernelConfig, PersistentKernelHandle, boot};

fn persistent_kernel(
    pool: retailer_sourcing::io::DbPool,
) -> Result<PersistentKernelHandle, Box<dyn std::error::Error + Send + Sync>> {
    Ok(boot(KernelConfig::default()).start_persistent(pool, 0)?)
}

const MAX_COMMANDS_PER_RUN: usize = 10;

#[derive(QueryableByName)]
struct ReservedCommandCount {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

fn commands_to_process_count(
    pool: &retailer_sourcing::io::DbPool,
    spec: &ReservableCommandSpec,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get()?;
    let count = sql_query(
        "SELECT COUNT(*) AS count
         FROM (
             SELECT 1
             FROM command_entries
             WHERE status IN (0, 4)
               AND scheduled_at <= NOW()
               AND attempts < $1
             ORDER BY scheduled_at ASC
             LIMIT $2
         ) eligible",
    )
    .bind::<Integer, _>(spec.max_attempts)
    .bind::<Integer, _>(spec.limit as i32)
    .get_result::<ReservedCommandCount>(&mut conn)?
    .count;

    Ok(count as usize)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = common::database_pool()?;
    let spec = ReservableCommandSpec::new(MAX_COMMANDS_PER_RUN);
    let processed_count = commands_to_process_count(&pool, &spec)?;
    let kernel = persistent_kernel(pool)?;

    let result = kernel.command_consumer().consume(&spec);

    match result {
        Ok(()) => println!("Processed {processed_count} commands."),
        Err(errors) => {
            common::print_batch_errors("command processing", &errors);
            println!(
                "Command processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}
