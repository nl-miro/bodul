mod common;

use kernel::io::{
    CommandGateway,
    CommandRecorder,
    CommandRecorderRepository,
    CommandStoreStorage, //
};
use kernel::io::{
    InboxConsumer, InboxConsumerRepository, InboxConsumerStorage, ReservableInboxSpec,
};
use std::sync::Arc;

const MAX_INBOX_MESSAGES_PER_RUN: usize = 10;

fn inbox_consumer(pool: retailer_sourcing::io::DbPool) -> InboxConsumer {
    let inbox_storage = Arc::new(InboxConsumerStorage::new(pool.clone()));
    let inbox_repository = InboxConsumerRepository::new(inbox_storage.clone(), inbox_storage);

    let command_store = Arc::new(CommandStoreStorage::new(pool));
    let command_recorder = Arc::new(CommandRecorder::new(Arc::new(
        CommandRecorderRepository::new(command_store),
    )));
    let command_gateway = CommandGateway::two_phased(command_recorder);

    InboxConsumer::new(inbox_repository, command_gateway)
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = common::database_pool()?;
    let consumer = inbox_consumer(pool);

    consumer
        .process(&ReservableInboxSpec::new(MAX_INBOX_MESSAGES_PER_RUN))
        .map_err(|errors| common::batch_error("inbox processing", errors))?;

    println!("Processed up to {MAX_INBOX_MESSAGES_PER_RUN} inbox messages.");

    Ok(())
}
