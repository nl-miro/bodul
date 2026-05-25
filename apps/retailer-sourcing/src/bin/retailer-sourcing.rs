use poem::{Server, listener::TcpListener};
use retailer_sourcing::app;

const SERVICE_URL: &str = "http://127.0.0.1:3001";
const BIND_ADDRESS: &str = "127.0.0.1:3001";

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    println!("Retailer Sourcing service listening on {}", SERVICE_URL);
    Server::new(TcpListener::bind(BIND_ADDRESS))
        .run(app())
        .await
}
