mod utils;

use chrono::Utc;
use diesel::OptionalExtension;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Nullable, Text, Uuid as SqlUuid};
use kernel::{NewCommandEnvelope, NewCommandMetadata};
use retailer_sourcing::io::{
    ActiveRetailers, RetailerSourcingEvent, SourceRetailer, SourcingMethod, StartDailySourcing,
};
use serial_test::serial;
use std::collections::HashSet;
use std::time::Duration;
use tokio::task::spawn_blocking;
use uuid::Uuid;

use crate::utils::{setup_full_kernel, wait_for};

const STATUS_COMPLETED: i32 = 5;
const WAIT_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Debug, QueryableByName)]
struct StatusRow {
    #[diesel(sql_type = Integer)]
    status: i32,
}

#[derive(Debug, QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(Debug, QueryableByName)]
struct EventRow {
    #[diesel(sql_type = SqlUuid)]
    id: Uuid,
    #[diesel(sql_type = Text)]
    payload: String,
}

#[derive(Debug, QueryableByName)]
struct SourceRetailerRow {
    #[diesel(sql_type = Text)]
    payload: String,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    correlation_id: Option<Uuid>,
    #[diesel(sql_type = Nullable<SqlUuid>)]
    causation_id: Option<Uuid>,
}

fn dispatch_start_daily_sourcing(
    state: std::sync::Arc<kernel::PersistentKernelState>,
    command_id: Uuid,
) -> tokio::task::JoinHandle<Result<(), kernel::KernelError>> {
    let envelope = NewCommandEnvelope {
        command: StartDailySourcing::new(Utc::now()),
        metadata: NewCommandMetadata {
            command_id,
            correlation_id: Some(command_id),
            causation_id: None,
            source: Some("test".to_string()),
        },
    };
    spawn_blocking(move || state.dispatch_command(envelope))
}

#[tokio::test]
#[serial]
async fn start_daily_sourcing_is_consumed_and_emits_daily_sourcing_requested() {
    let kernel = setup_full_kernel();
    let command_id = Uuid::now_v7();
    dispatch_start_daily_sourcing(kernel.state.clone(), command_id)
        .await
        .unwrap()
        .unwrap();

    let pool = kernel.pool.clone();
    wait_for(WAIT_TIMEOUT, || {
        let mut conn = pool.get().unwrap();
        let row: StatusRow = sql_query("SELECT status FROM command_entries WHERE id = $1")
            .bind::<SqlUuid, _>(command_id)
            .get_result(&mut conn)
            .unwrap();
        row.status == STATUS_COMPLETED
    })
    .await;

    let mut conn = kernel.pool.get().unwrap();
    let events: Vec<EventRow> = sql_query(
        "SELECT id, payload FROM event_entries \
         WHERE event_type = 'DailySourcingRequested' \
         AND (meta->>'causation_id')::uuid = $1",
    )
    .bind::<SqlUuid, _>(command_id)
    .load(&mut conn)
    .unwrap();
    assert_eq!(
        events.len(),
        1,
        "exactly one DailySourcingRequested event caused by this command",
    );
    let wrapped: RetailerSourcingEvent = serde_json::from_str(&events[0].payload).unwrap();
    let RetailerSourcingEvent::DailySourcingRequested(event) = wrapped;
    assert_eq!(event.retailer_codes, ActiveRetailers::list());

    drop(conn);
    kernel.shutdown().await;
}

#[tokio::test]
#[serial]
async fn fan_out_produces_one_source_retailer_per_retailer_method_pair() {
    let kernel = setup_full_kernel();
    let command_id = Uuid::now_v7();
    dispatch_start_daily_sourcing(kernel.state.clone(), command_id)
        .await
        .unwrap()
        .unwrap();

    let expected = ActiveRetailers::list().len() * SourcingMethod::all().len();
    let pool = kernel.pool.clone();
    wait_for(WAIT_TIMEOUT, || {
        let mut conn = pool.get().unwrap();
        let row: Option<StatusRow> = sql_query(
            "SELECT status FROM event_entries \
             WHERE event_type = 'DailySourcingRequested' \
             AND (meta->>'causation_id')::uuid = $1 \
             LIMIT 1",
        )
        .bind::<SqlUuid, _>(command_id)
        .get_result(&mut conn)
        .optional()
        .unwrap();
        matches!(row, Some(r) if r.status == STATUS_COMPLETED)
    })
    .await;

    let mut conn = kernel.pool.get().unwrap();
    let count: CountRow = sql_query(
        "SELECT COUNT(*) AS count FROM command_entries \
         WHERE command_type = 'SourceRetailer' \
         AND (meta->>'correlation_id')::uuid = $1",
    )
    .bind::<SqlUuid, _>(command_id)
    .get_result(&mut conn)
    .unwrap();
    assert_eq!(count.count as usize, expected);

    let rows: Vec<SourceRetailerRow> = sql_query(
        "SELECT payload, \
                (meta->>'correlation_id')::uuid AS correlation_id, \
                (meta->>'causation_id')::uuid AS causation_id \
         FROM command_entries \
         WHERE command_type = 'SourceRetailer' \
         AND (meta->>'correlation_id')::uuid = $1",
    )
    .bind::<SqlUuid, _>(command_id)
    .load(&mut conn)
    .unwrap();
    let pairs: HashSet<(String, SourcingMethod)> = rows
        .iter()
        .map(|r| {
            let cmd: SourceRetailer = serde_json::from_str(&r.payload).unwrap();
            (cmd.retailer_code.as_string().to_string(), cmd.method)
        })
        .collect();
    for code in ActiveRetailers::list() {
        for method in SourcingMethod::all() {
            assert!(
                pairs.contains(&(code.as_string().to_string(), *method)),
                "missing pair ({}, {:?})",
                code.as_string(),
                method,
            );
        }
    }

    drop(conn);
    kernel.shutdown().await;
}

#[tokio::test]
#[serial]
async fn source_retailer_commands_carry_causation_and_correlation_chain() {
    let kernel = setup_full_kernel();
    let command_id = Uuid::now_v7();
    dispatch_start_daily_sourcing(kernel.state.clone(), command_id)
        .await
        .unwrap()
        .unwrap();

    let expected = ActiveRetailers::list().len() * SourcingMethod::all().len();
    let pool = kernel.pool.clone();
    wait_for(WAIT_TIMEOUT, || {
        let mut conn = pool.get().unwrap();
        let row: CountRow = sql_query(
            "SELECT COUNT(*) AS count FROM command_entries \
             WHERE command_type = 'SourceRetailer' \
             AND (meta->>'correlation_id')::uuid = $1",
        )
        .bind::<SqlUuid, _>(command_id)
        .get_result(&mut conn)
        .unwrap();
        row.count as usize == expected
    })
    .await;

    let mut conn = kernel.pool.get().unwrap();
    let event: EventRow = sql_query(
        "SELECT id, payload FROM event_entries \
         WHERE event_type = 'DailySourcingRequested' \
         AND (meta->>'causation_id')::uuid = $1",
    )
    .bind::<SqlUuid, _>(command_id)
    .get_result(&mut conn)
    .unwrap();

    let rows: Vec<SourceRetailerRow> = sql_query(
        "SELECT payload, \
                (meta->>'correlation_id')::uuid AS correlation_id, \
                (meta->>'causation_id')::uuid AS causation_id \
         FROM command_entries \
         WHERE command_type = 'SourceRetailer' \
         AND (meta->>'correlation_id')::uuid = $1",
    )
    .bind::<SqlUuid, _>(command_id)
    .load(&mut conn)
    .unwrap();
    assert_eq!(rows.len(), expected);
    for row in &rows {
        assert_eq!(
            row.causation_id,
            Some(event.id),
            "SourceRetailer causation_id should match DailySourcingRequested event_id",
        );
        assert_eq!(
            row.correlation_id,
            Some(command_id),
            "SourceRetailer correlation_id should propagate from StartDailySourcing",
        );
    }

    drop(conn);
    kernel.shutdown().await;
}
