use poem::{get, web::Json, App, IntoResponse, Route};
use serde::Serialize;

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    service: String,
}

#[get("/health")]
fn health_check() -> impl IntoResponse {
    Json(HealthCheck {
        status: "ok".to_string(),
        service: "retailer-sourcing".to_string(),
    })
}

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let app = App::new().route(Route::new().at("/health", get(health_check)));

    println!("Retailer Sourcing service listening on http://127.0.0.1:3001");
    poem::Server::new(poem::listener::TcpListener::bind("127.0.0.1:3001"))
        .run(app)
        .await
}
