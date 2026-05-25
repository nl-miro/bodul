mod common;

use kernel::io::ReservableCommandSpec;
use kernel::{KernelConfig, PersistentKernelHandle, boot};

fn persistent_kernel(
    pool: retailer_sourcing::io::DbPool,
) -> Result<PersistentKernelHandle, Box<dyn std::error::Error + Send + Sync>> {
    Ok(boot(KernelConfig::default()).start_persistent(pool, 0)?)
}

const MAX_COMMANDS_PER_RUN: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = common::database_pool()?;
    let kernel = persistent_kernel(pool)?;

    kernel
        .command_consumer()
        .consume(&ReservableCommandSpec::new(MAX_COMMANDS_PER_RUN))
        .map_err(|errors| common::batch_error("command processing", errors))?;

    println!("Processed up to {MAX_COMMANDS_PER_RUN} commands.");

    Ok(())
}
