#![allow(dead_code)]

use dotenvy::dotenv;
use kernel::io::{
    CommandGateway, CommandRecorder, CommandRecorderRepository, CommandStoreStorage, InboxConsumer,
    InboxConsumerRepository, InboxConsumerStorage, ReservableCommandSpec, ReservableEventSpec,
    ReservableInboxSpec,
};
use kernel::{KernelConfig, boot};
use lapin::{Connection, ConnectionProperties};
use outbox::io::{
    AmqpPublishConfig, AmqpPublisher, OutboxConsumer, ReservableOutboxSpec, consumer_repository,
};
use retailer_sourcing::io::{DbPool, build_pool};
use retailer_sourcing::run_migrations;
use std::env::var;
use std::fmt::Display;
use std::io::Error;
use std::sync::Arc;

const MAX_INBOX_MESSAGES_PER_RUN: usize = 10;
const MAX_COMMANDS_PER_RUN: usize = 10;
const MAX_EVENTS_PER_RUN: usize = 10;
const MAX_OUTBOX_MESSAGES_PER_RUN: usize = 10;

pub fn database_pool() -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    database_pool_from_url(&database_url)
}

pub fn database_pool_from_url(
    database_url: &str,
) -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let pool = build_pool(database_url)?;
    run_migrations(&pool)?;
    Ok(pool)
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

pub fn batch_error<T>(kind: &str, errors: Vec<T>) -> Error
where
    T: Display,
{
    print_batch_errors(kind, &errors);

    Error::other(format!("failed to process {kind}"))
}

pub fn print_batch_errors<T>(kind: &str, errors: &[T])
where
    T: Display,
{
    for error in errors {
        eprintln!("{kind}: {error}");
    }
}

pub fn run_inbox(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let inbox_storage = Arc::new(InboxConsumerStorage::new(pool.clone()));
    let inbox_repository = InboxConsumerRepository::new(inbox_storage.clone(), inbox_storage);

    let command_store = Arc::new(CommandStoreStorage::new(pool));
    let command_recorder = Arc::new(CommandRecorder::new(Arc::new(
        CommandRecorderRepository::new(command_store),
    )));
    let command_gateway = CommandGateway::two_phased(command_recorder);

    let consumer = InboxConsumer::new(inbox_repository, command_gateway);
    let result = consumer.process(&ReservableInboxSpec::new(MAX_INBOX_MESSAGES_PER_RUN));

    match result {
        Ok(processed_count) => println!("Processed {processed_count} inbox messages."),
        Err(errors) => {
            print_batch_errors("inbox processing", &errors);
            println!(
                "Inbox processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}

pub fn run_commands(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kernel = boot(KernelConfig::default()).start_persistent(pool, 0)?;
    let result = kernel
        .command_consumer()
        .consume(&ReservableCommandSpec::new(MAX_COMMANDS_PER_RUN));

    match result {
        Ok(processed_count) => println!("Processed {processed_count} commands."),
        Err(errors) => {
            print_batch_errors("command processing", &errors);
            println!(
                "Command processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}

pub fn run_events(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let kernel = boot(KernelConfig::default()).start_persistent(pool, 0)?;
    let result = kernel
        .event_consumer()
        .consume(&ReservableEventSpec::new(MAX_EVENTS_PER_RUN));

    match result {
        Ok(processed_count) => println!("Processed {processed_count} events."),
        Err(errors) => {
            print_batch_errors("event processing", &errors);
            println!(
                "Event processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}

pub fn run_outbox(pool: DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let repository = consumer_repository(pool);
    let publisher = Arc::new(amqp_publisher()?);
    let consumer = OutboxConsumer::new(repository, publisher);

    let result = consumer.publish_batch(&ReservableOutboxSpec::new(MAX_OUTBOX_MESSAGES_PER_RUN));

    match result {
        Ok(processed_count) => println!("Processed {processed_count} outbox messages."),
        Err(errors) => {
            print_batch_errors("outbox processing", &errors);
            println!(
                "Outbox processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}

fn amqp_publisher() -> Result<AmqpPublisher, Box<dyn std::error::Error + Send + Sync>> {
    let amqp_url = var("AMQP_URL").expect("AMQP_URL must be set");
    let exchange = var("OUTBOX_EXCHANGE").unwrap_or_default();
    let default_content_type =
        var("OUTBOX_CONTENT_TYPE").unwrap_or_else(|_| "application/json".to_string());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let channel = runtime.block_on(async {
        let connection = Connection::connect(&amqp_url, ConnectionProperties::default()).await?;
        connection.create_channel().await
    })?;

    Ok(AmqpPublisher::new(
        channel,
        AmqpPublishConfig {
            exchange,
            mandatory: false,
            default_content_type,
        },
    ))
}
