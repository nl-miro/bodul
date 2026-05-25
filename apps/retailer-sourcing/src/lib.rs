// Retailer Sourcing service library

mod app_event;
mod assembly;
mod check_health;
mod daily_sourcing;
mod entry_error_tracing;
mod source_retailer;

use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use kernel::PersistentKernelState;
use poem::Route;
use std::sync::Arc;

pub mod io {
    pub use crate::app_event::io::*;
    pub use crate::assembly::io::*;
    pub use crate::check_health::io::*;
    pub use crate::daily_sourcing::io::*;
    pub use crate::entry_error_tracing::io::*;
    pub use crate::source_retailer::io::*;

    use diesel::PgConnection;
    use diesel::r2d2::{ConnectionManager, Pool, PoolError};

    pub type DbPool = Pool<ConnectionManager<PgConnection>>;

    pub fn build_pool(database_url: &str) -> Result<DbPool, PoolError> {
        kernel::io::build_pool(database_url)
    }
}

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn run_migrations(pool: &io::DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS)?;
    Ok(())
}

pub fn app(state: Arc<PersistentKernelState>, bearer: Arc<String>) -> Route {
    let route = Route::new();
    let route = check_health::io::register(route);
    daily_sourcing::io::register(route, state, bearer)
}
