pub use super::application::start_mulac;
use kernel::EventGateway;
use kernel::io::{
    CommandConsumer, CommandConsumerRepository, CommandConsumerStorage, CommandDispatcher,
    CommandHandlerRegistry, DbPool, EventRecorder, EventRecorderRepository, EventStoreStorage,
};
use std::sync::Arc;

struct Dic {
    db_pool: DbPool,
}

impl Dic {
    pub fn new(db_pool: DbPool) -> Self {
        Self { db_pool }
    }

    pub fn command_consumer(&self) -> CommandConsumer {
        let command_storage = Arc::new(CommandConsumerStorage::new(self.db_pool.clone()));
        let command_repository =
            CommandConsumerRepository::new(command_storage.clone(), command_storage);

        let event_store = Arc::new(EventStoreStorage::new(self.db_pool.clone()));
        let event_recorder = Arc::new(EventRecorder::new(Arc::new(EventRecorderRepository::new(
            event_store,
        ))));
        let event_gateway = Arc::new(EventGateway::two_phased(event_recorder));

        let command_registry = Arc::new(CommandHandlerRegistry::from_handlers(Vec::new()));
        let command_dispatcher = Arc::new(CommandDispatcher::new(command_registry, event_gateway));

        CommandConsumer::new(command_repository, command_dispatcher)
    }
}
