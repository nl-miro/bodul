pub mod io {
    pub use super::http::{HEALTH_CHECK_PATH, health_check, register};
}

mod domain {}

mod http {
    use poem::{
        Route,
        get,
        handler,
        web::Json, //
    };
    use serde::Serialize;

    const SERVICE_NAME: &str = "retailer-sourcing";
    const HEALTH_STATUS_OK: &str = "ok";

    pub const HEALTH_CHECK_PATH: &str = "/health";

    #[derive(Debug, Serialize)]
    pub struct HealthCheck {
        status: String,
        service: String,
    }

    #[handler]
    pub fn health_check() -> Json<HealthCheck> {
        Json(HealthCheck {
            status: HEALTH_STATUS_OK.to_string(),
            service: SERVICE_NAME.to_string(),
        })
    }

    pub fn register(route: Route) -> Route {
        route.at(HEALTH_CHECK_PATH, get(health_check))
    }
}
