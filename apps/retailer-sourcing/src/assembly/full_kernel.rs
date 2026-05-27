use super::dic::start_mulac;
use crate::io::{DbPool, build_pool};
use crate::run_migrations;
use dotenvy::dotenv;
use kernel::io::{run_command_worker, run_event_worker};
use kernel::{PersistentKernelHandle, PersistentKernelState};
use std::env::var;
use std::sync::{Arc, OnceLock};
use tokio::task::JoinHandle;

const TEST_DATABASE_URL_ENV: &str = "TEST_DATABASE_URL";
const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://postgres:postgres@localhost:26001/retailer_sourcing_test";
const TEST_LOG_FILE_NAME: &str = "retailer-sourcing-tests.log";

pub struct FullKernel {
    pub pool: DbPool,
    pub state: Arc<PersistentKernelState>,
    pub handle: PersistentKernelHandle,
    pub command_worker: JoinHandle<()>,
    pub event_worker: JoinHandle<()>,
}

impl FullKernel {
    pub async fn shutdown(self) {
        self.handle.shutdown();
        let _ = self.command_worker.await;
        let _ = self.event_worker.await;
    }
}

pub fn setup_full_kernel() -> FullKernel {
    setup_full_kernel_with_pool(shared_pool())
}

pub fn setup_full_kernel_with_pool(pool: DbPool) -> FullKernel {
    run_migrations(&pool).expect("migrations failed");
    let handle = start_mulac(pool.clone()).expect("failed to start mulac");
    let command_worker = tokio::spawn(run_command_worker(
        handle.command_consumer(),
        handle.child_token(),
    ));
    let event_worker = tokio::spawn(run_event_worker(
        handle.event_consumer(),
        handle.child_token(),
    ));
    let state = Arc::new(handle.state());
    FullKernel {
        pool,
        state,
        handle,
        command_worker,
        event_worker,
    }
}

pub fn shared_pool() -> DbPool {
    static POOL: OnceLock<DbPool> = OnceLock::new();
    POOL.get_or_init(|| {
        dotenv().ok();
        let url =
            var(TEST_DATABASE_URL_ENV).unwrap_or_else(|_| DEFAULT_TEST_DATABASE_URL.to_string());
        build_pool(&url).expect("failed to build test pool")
    })
    .clone()
}
