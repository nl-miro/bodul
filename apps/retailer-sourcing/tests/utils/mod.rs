#![allow(dead_code)]

use diesel::prelude::*;
use diesel::sql_query;
use dotenvy::dotenv;
use kernel::{KernelConfig, PersistentKernelHandle, PersistentKernelState, boot};
use retailer_sourcing::io::{
    DbPool, build_pool, run_command_worker, run_event_worker, start_mulac,
};
use retailer_sourcing::run_migrations;
use std::env::var;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

const TEST_DATABASE_URL_ENV: &str = "TEST_DATABASE_URL";
const DEFAULT_TEST_DATABASE_URL: &str =
    "postgres://postgres:postgres@localhost:26001/retailer_sourcing_test";
const TEST_LOG_FILE_NAME: &str = "retailer-sourcing-tests.log";

fn init_test_logging() {
    static LOG_GUARD: OnceLock<WorkerGuard> = OnceLock::new();

    LOG_GUARD.get_or_init(|| {
        let file_appender = tracing_appender::rolling::never("target", TEST_LOG_FILE_NAME);
        let (writer, guard) = tracing_appender::non_blocking(file_appender);
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("error"));

        tracing_subscriber::fmt()
            .with_ansi(false)
            .with_env_filter(filter)
            .with_writer(writer)
            .try_init()
            .expect("failed to initialize test tracing");

        let default_panic_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            tracing::error!("test panic: {panic_info}");
            default_panic_hook(panic_info);
        }));

        guard
    });
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

const KEEP_DB_ENV: &str = "RETAILER_SOURCING_TEST_KEEP_DB";

fn should_keep_db() -> bool {
    static KEEP: OnceLock<bool> = OnceLock::new();
    *KEEP.get_or_init(|| {
        dotenv().ok();
        let keep = matches!(
            var(KEEP_DB_ENV)
                .ok()
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref(),
            Some("1" | "true" | "yes"),
        );
        if keep {
            eprintln!("retailer-sourcing tests: KEEP_DB mode — tables will not be truncated",);
        }
        keep
    })
}

pub fn reset_tables(pool: &DbPool) {
    let mut conn = pool.get().unwrap();
    sql_query(
        "TRUNCATE TABLE command_entries, event_entries, inbox_entries, outbox_entries \
         RESTART IDENTITY CASCADE",
    )
    .execute(&mut conn)
    .unwrap();
}

pub fn reset_tables_if_needed(pool: &DbPool) {
    if should_keep_db() {
        return;
    }
    reset_tables(pool);
}

pub fn setup() -> (DbPool, Arc<PersistentKernelState>) {
    init_test_logging();
    let pool = shared_pool();
    run_migrations(&pool).expect("migrations failed");
    reset_tables_if_needed(&pool);
    let handle = boot(KernelConfig::default())
        .start_persistent(pool.clone(), 0)
        .expect("failed to start kernel");
    let state = Arc::new(handle.state());
    (pool, state)
}

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
    init_test_logging();
    let pool = shared_pool();
    run_migrations(&pool).expect("migrations failed");
    reset_tables_if_needed(&pool);
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

pub async fn wait_for<F>(timeout: Duration, mut predicate: F)
where
    F: FnMut() -> bool,
{
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if predicate() {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "wait_for: timed out after {timeout:?}",
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}
