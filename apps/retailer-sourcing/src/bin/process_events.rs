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

    let result = kernel
        .event_consumer()
        .consume(&ReservableEventSpec::new(MAX_EVENTS_PER_RUN));

    match result {
        Ok(()) => println!("Processed up to {MAX_EVENTS_PER_RUN} events."),
        Err(errors) => {
            common::print_batch_errors("event processing", &errors);
            println!(
                "Event processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}
