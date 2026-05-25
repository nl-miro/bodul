mod common;

use lapin::{Connection, ConnectionProperties};
use outbox::io::{
    AmqpPublishConfig, AmqpPublisher, OutboxConsumer, ReservableOutboxSpec, consumer_repository,
};
use std::env::var;
use std::sync::Arc;

const MAX_OUTBOX_MESSAGES_PER_RUN: usize = 10;

fn outbox_consumer(
    pool: retailer_sourcing::io::DbPool,
) -> Result<OutboxConsumer, Box<dyn std::error::Error + Send + Sync>> {
    let repository = consumer_repository(pool);
    let publisher = Arc::new(amqp_publisher()?);
    Ok(OutboxConsumer::new(repository, publisher))
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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let pool = common::database_pool()?;
    let consumer = outbox_consumer(pool)?;

    let result = consumer.publish_batch(&ReservableOutboxSpec::new(MAX_OUTBOX_MESSAGES_PER_RUN));

    match result {
        Ok(()) => println!("Processed up to {MAX_OUTBOX_MESSAGES_PER_RUN} outbox messages."),
        Err(errors) => {
            common::print_batch_errors("outbox processing", &errors);
            println!(
                "Outbox processing finished with {} error(s); see stderr for details.",
                errors.len()
            );
        }
    }

    Ok(())
}
