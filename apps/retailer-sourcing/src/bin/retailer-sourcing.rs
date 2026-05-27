use dotenvy::dotenv;
use poem::{Server, listener::TcpListener};
use retailer_sourcing::io::{build_pool, run_command_worker, run_event_worker, start_mulac};
use retailer_sourcing::{app, run_migrations};
use std::env::var;
use std::sync::Arc;

const BIND_ADDRESS: &str = "127.0.0.1:3001";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    dotenv().ok();

    let database_url = var("DATABASE_URL").expect("DATABASE_URL must be set");
    let bearer_token =
        var("RETAILER_SOURCING_BEARER_TOKEN").expect("RETAILER_SOURCING_BEARER_TOKEN must be set");

    let pool = build_pool(&database_url)?;
    run_migrations(&pool)?;

    let handle = start_mulac(pool)?;

    let command_worker = tokio::spawn(run_command_worker(
        handle.command_consumer(),
        handle.child_token(),
    ));
    let event_worker = tokio::spawn(run_event_worker(
        handle.event_consumer(),
        handle.child_token(),
    ));

    let state = Arc::new(handle.state());
    let bearer = Arc::new(bearer_token);

    println!("Retailer Sourcing service listening on http://{BIND_ADDRESS}");
    tokio::select! {
        res = Server::new(TcpListener::bind(BIND_ADDRESS)).run(app(state, bearer)) => {
            if let Err(e) = res {
                eprintln!("HTTP server error: {e}");
            }
        }
        res = command_worker => {
            eprintln!("Command worker exited unexpectedly: {res:?}");
        }
        res = event_worker => {
            eprintln!("Event worker exited unexpectedly: {res:?}");
        }
        _ = tokio::signal::ctrl_c() => {
            println!("Received SIGINT, shutting down...");
        }
    }
    handle.shutdown();
    handle.wait().await?;

    Ok(())
}
