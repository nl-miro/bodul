use poem::{get, web::Json, App, IntoResponse, Route};
use serde::Serialize;

const SERVICE_NAME: &str = "retailer-sourcing";
const SERVICE_URL: &str = "http://127.0.0.1:3001";
const BIND_ADDRESS: &str = "127.0.0.1:3001";
const HEALTH_CHECK_PATH: &str = "/health";
const HEALTH_STATUS_OK: &str = "ok";

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    service: String,
}

#[get("/health")]
fn health_check() -> impl IntoResponse {
    Json(HealthCheck {
        status: HEALTH_STATUS_OK.to_string(),
        service: SERVICE_NAME.to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = App::new().route(Route::new().at(HEALTH_CHECK_PATH, get(health_check)));

    println!("Retailer Sourcing service listening on {}", SERVICE_URL);
    poem::Server::new(poem::listener::TcpListener::bind(BIND_ADDRESS))
        .run(app)
        .await
}
