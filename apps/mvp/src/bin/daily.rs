//! Enqueues daily sitemap retrieval requests for every configured retailer.

use std::error::Error;

use ::retailer_sourcing::registry::sitemap_config;
use kernel::io::NewCommandMetadata;
use mvp::assembly::boot;
use mvp::assembly::io::{AppCommand, NewCommandEnvelope};
use mvp::sitemap_discovery::io::RequestSitemapRetrieval;
use bodul_shared::retailer::RetailerCode;
use uuid::Uuid;

fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    let _guard = mvp::logging::init();
    let kernel = boot()?;

    let state = kernel.state();

    let mut enqueued = 0usize;
    let mut skipped = 0usize;

    // let retailer_codes = RetailerCode::ALL;
    // TODO
    let retailer_codes = vec![RetailerCode::AnkerCa];

    for retailer_code in retailer_codes {
        if sitemap_config(&retailer_code).is_none() {
            skipped += 1;
            continue;
        }

        let envelope = create_command(retailer_code);

        state.dispatch_command(envelope)?;

        enqueued += 1;
    }

    println!("done: {enqueued} enqueued, {skipped} skipped");
    Ok(())
}

fn create_command(retailer_code: RetailerCode) -> NewCommandEnvelope {
    let command_id = Uuid::now_v7();

    let cmd = RequestSitemapRetrieval::new(command_id, retailer_code);
    let command = AppCommand::RequestSitemapRetrieval(cmd);

    let metadata = NewCommandMetadata {
        command_id,
        correlation_id: Some(command_id),
        causation_id: None,
        source: Some("mvp.daily".to_string()),
    };

    NewCommandEnvelope { command, metadata }
}
