use crate::daily_sourcing;
use crate::source_retailer;
use kernel::io::DbPool;
use kernel::{EventSubscriberPort, KernelConfig, KernelError, PersistentKernelHandle, boot};
use std::sync::Arc;

pub fn start_mulac(pool: DbPool) -> Result<PersistentKernelHandle, KernelError> {
    boot(KernelConfig::default())
        .command_handler(
            daily_sourcing::io::COMMAND_TYPE,
            Arc::new(daily_sourcing::io::DailySourcingHandler::new()),
        )
        .event_subscriber_with_command_gateway(
            daily_sourcing::io::EVENT_TYPE_DAILY_SOURCING_REQUESTED,
            "daily-sourcing-fan-out",
            |command_gateway| {
                Arc::new(source_retailer::io::DailySourcingFanOutSubscriber::new(
                    command_gateway,
                )) as Arc<dyn EventSubscriberPort>
            },
        )
        .start_persistent(pool, 0)
}
