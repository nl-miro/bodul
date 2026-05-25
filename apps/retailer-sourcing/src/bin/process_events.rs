mod common;

use kernel::io::ReservableEventSpec;
use kernel::{KernelConfig, PersistentKernelHandle, boot};

fn persistent_kernel(
    pool: retailer_sourcing::io::DbPool,
) -> Result<PersistentKernelHandle, Box<dyn std::error::Error + Send + Sync>> {
    Ok(boot(KernelConfig::default()).start_persistent(pool, 0)?)
}

const MAX_EVENTS_PER_RUN: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = common::database_pool()?;
    let kernel = persistent_kernel(pool)?;

    kernel
        .event_consumer()
        .consume(&ReservableEventSpec::new(MAX_EVENTS_PER_RUN))
        .map_err(|errors| common::batch_error("event processing", errors))?;

    println!("Processed up to {MAX_EVENTS_PER_RUN} events.");

    Ok(())
}
