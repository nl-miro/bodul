mod utils;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Integer, Text, Uuid as SqlUuid};
use kernel::PersistentKernelState;
use poem::Endpoint;
use poem::http::StatusCode;
use poem::test::TestClient;
use retailer_sourcing::{app, io::DAILY_SOURCING_PATH};
use serde_json::{Value, from_value};
use serial_test::serial;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, QueryableByName)]
struct CommandEntryRow {
    #[diesel(sql_type = Integer)]
    status: i32,
    #[diesel(sql_type = Text)]
    command_type: String,
}

#[derive(Debug, QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

fn make_app(state: Arc<PersistentKernelState>, bearer: &str) -> impl Endpoint {
    app(state, Arc::new(bearer.to_string()))
}

#[tokio::test]
#[serial]
async fn returns_401_when_no_token_provided() {
    let (_, state) = utils::setup();
    let client = TestClient::new(make_app(state, "secret"));

    let resp = client.post(DAILY_SOURCING_PATH).send().await;

    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn returns_401_when_wrong_token_provided() {
    let (_, state) = utils::setup();
    let client = TestClient::new(make_app(state, "secret"));

    let resp = client
        .post(DAILY_SOURCING_PATH)
        .header("Authorization", "Bearer wrong-token")
        .send()
        .await;

    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn enqueues_command_and_returns_202_with_command_id() {
    let (pool, state) = utils::setup();
    let bearer = "test-bearer-token";
    let client = TestClient::new(make_app(state, bearer));

    let resp = client
        .post(DAILY_SOURCING_PATH)
        .header("Authorization", format!("Bearer {bearer}"))
        .send()
        .await;

    resp.assert_status(StatusCode::ACCEPTED);

    let body: Value = resp.json().await.value().deserialize();
    let command_id: Uuid = from_value(body["command_id"].clone())
        .expect("response should have a valid command_id UUID");

    // Verify the command was durably written to command_entries
    let mut conn = pool.get().unwrap();
    let row = sql_query("SELECT status, command_type FROM command_entries WHERE id = $1")
        .bind::<SqlUuid, _>(command_id)
        .get_result::<CommandEntryRow>(&mut conn)
        .expect("command entry should exist in the database");

    assert_eq!(
        row.status, 0,
        "status should be 0 (Received/not yet processed)"
    );
    assert_eq!(row.command_type, "StartDailySourcing");
}

#[tokio::test]
#[serial]
async fn second_request_enqueues_a_distinct_command() {
    let (pool, state) = utils::setup();
    let bearer = "test-bearer-token";
    let client = TestClient::new(make_app(state, bearer));

    let auth = format!("Bearer {bearer}");

    let resp1 = client
        .post(DAILY_SOURCING_PATH)
        .header("Authorization", auth.clone())
        .send()
        .await;
    resp1.assert_status(StatusCode::ACCEPTED);
    let body1: Value = resp1.json().await.value().deserialize();

    let resp2 = client
        .post(DAILY_SOURCING_PATH)
        .header("Authorization", auth)
        .send()
        .await;
    resp2.assert_status(StatusCode::ACCEPTED);
    let body2: Value = resp2.json().await.value().deserialize();

    let id1: Uuid = from_value(body1["command_id"].clone()).unwrap();
    let id2: Uuid = from_value(body2["command_id"].clone()).unwrap();
    assert_ne!(
        id1, id2,
        "each trigger should produce a distinct command_id"
    );

    let mut conn = pool.get().unwrap();
    let row = sql_query(
        "SELECT COUNT(*) AS count FROM command_entries \
         WHERE id IN ($1, $2) AND command_type = 'StartDailySourcing'",
    )
    .bind::<SqlUuid, _>(id1)
    .bind::<SqlUuid, _>(id2)
    .get_result::<CountRow>(&mut conn)
    .expect("count query should succeed");
    assert_eq!(row.count, 2, "both dispatched commands should be persisted");
}
