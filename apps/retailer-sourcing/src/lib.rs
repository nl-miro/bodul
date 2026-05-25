// Retailer Sourcing service library

mod check_health;

use poem::Route;

pub mod io {
    pub use crate::check_health::io::*;
}

pub fn app() -> Route {
    check_health::route()
}
