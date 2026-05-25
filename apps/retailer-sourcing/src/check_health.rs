use poem::{Route, get, handler, web::Json};
use serde::Serialize;

const SERVICE_NAME: &str = "retailer-sourcing";
const HEALTH_STATUS_OK: &str = "ok";

pub mod io {
    pub const HEALTH_CHECK_PATH: &str = "/health";
}

#[derive(Serialize)]
struct HealthCheck {
    status: String,
    service: String,
}

#[handler]
fn health_check() -> Json<HealthCheck> {
    Json(HealthCheck {
        status: HEALTH_STATUS_OK.to_string(),
        service: SERVICE_NAME.to_string(),
    })
}

pub fn route() -> Route {
    Route::new().at(io::HEALTH_CHECK_PATH, get(health_check))
}
