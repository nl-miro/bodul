mod utils;

use poem::test::TestClient;
use retailer_sourcing::{app, io::HEALTH_CHECK_PATH};
use std::sync::Arc;

#[tokio::test]
async fn health_endpoint_returns_ok() {
    let (_, state) = utils::setup();
    let client = TestClient::new(app(state, Arc::new("unused".to_string())));

    let resp = client.get(HEALTH_CHECK_PATH).send().await;
    resp.assert_status_is_ok();

    let body: serde_json::Value = resp.json().await.value().deserialize();
    assert_eq!(body["status"], "ok");
    assert_eq!(body["service"], "retailer-sourcing");
}
