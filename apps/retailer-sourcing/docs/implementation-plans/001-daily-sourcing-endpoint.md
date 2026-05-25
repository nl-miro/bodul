# 001 — Daily Sourcing Endpoint

## Context

Operators (or a scheduler) need a way to trigger the daily sourcing run on
demand. The retailer-sourcing service should expose a single HTTP endpoint
that, when called with a valid Bearer token, enqueues a `StartDailySourcing`
command. The command itself is not executed by the request — it is persisted
to the command queue (mulac's commanding store) and processed asynchronously
by a separate consumer. This separation keeps the HTTP request fast,
idempotent at the queue level, and resilient to handler crashes.

## Scope

- Add `POST /daily-sourcing/` to the existing Poem router.
- Reject requests without a valid `Authorization: Bearer <token>` header.
- Construct a `StartDailySourcing` command and hand it to a
  `CommandGateway` configured for the **two-phased** outbox pattern
  (`CommandGateway::two_phased`), so the command is durably recorded in the
  commands table before the HTTP response returns.

Out of scope for this plan: implementing the `StartDailySourcing` handler
itself, the consumer loop, and the downstream sourcing workflow. Those land
in follow-up plans.

## Design

### Feature module

Create a new private feature module under
`apps/retailer-sourcing/src/daily_sourcing.rs`, following the convention in
[docs/coding-guidelines.md](../../../../docs/coding-guidelines.md):

```text
src/
  daily_sourcing.rs        // handler + route + io submodule
```

Public surface (re-exported through `lib.rs`'s top-level `pub mod io`):

- `pub const DAILY_SOURCING_PATH: &str = "/daily-sourcing/";`
- `pub const COMMAND_TYPE: &str = "StartDailySourcing";`

Private items:

- `#[handler] async fn trigger(...)` — extracts the bearer token, builds a
  `NewCommandEnvelope`, calls `gateway.dispatch(envelope)`, returns
  `202 Accepted` with the generated `command_id` on success.
- `pub fn route(gateway: Arc<CommandGateway>) -> Route` — wires the handler
  with `Route::new().at(io::DAILY_SOURCING_PATH, post(trigger))` and attaches
  the gateway via `.with(AddData::new(gateway))` so the handler can pull it
  out with `Data(&Arc<CommandGateway>)`.

`lib.rs::app()` becomes:

```rust
pub fn app(gateway: Arc<CommandGateway>) -> Route {
    Route::new()
        .nest("/", check_health::route())
        .nest("/", daily_sourcing::route(gateway))
}
```

### Bearer authentication

Poem has no built-in bearer middleware, so we write a small one. Live it in
its own module — `apps/retailer-sourcing/src/bearer_auth.rs` — so it can be
attached to any route group, not just `daily-sourcing/`.

Sketch:

```rust
// src/bearer_auth.rs
use poem::{Endpoint, Middleware, Request, Result, http::StatusCode, error::Error};
use std::sync::Arc;

#[derive(Clone)]
pub struct BearerAuth {
    expected: Arc<String>,
}

impl BearerAuth {
    pub fn new(token: impl Into<String>) -> Self {
        Self { expected: Arc::new(token.into()) }
    }
}

impl<E: Endpoint> Middleware<E> for BearerAuth {
    type Output = BearerAuthEndpoint<E>;
    fn transform(&self, ep: E) -> Self::Output {
        BearerAuthEndpoint { inner: ep, expected: self.expected.clone() }
    }
}

pub struct BearerAuthEndpoint<E> {
    inner: E,
    expected: Arc<String>,
}

impl<E: Endpoint> Endpoint for BearerAuthEndpoint<E> {
    type Output = E::Output;
    async fn call(&self, req: Request) -> Result<Self::Output> {
        let provided = req
            .headers()
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        match provided {
            Some(token) if constant_time_eq(token.as_bytes(), self.expected.as_bytes()) => {
                self.inner.call(req).await
            }
            _ => Err(Error::from_status(StatusCode::UNAUTHORIZED)),
        }
    }
}
```

Use a constant-time comparison (`subtle::ConstantTimeEq` or a small local
helper) so the check doesn't leak the token via timing.

Attach it via `.with(BearerAuth::new(token))` on the protected route group:

```rust
Route::new()
    .at(io::DAILY_SOURCING_PATH, post(trigger))
    .with(BearerAuth::new(bearer_token))
```

The middleware module's `io` submodule exposes only `BearerAuth`; the
endpoint type stays private.

### Command envelope

```rust
let command_id = Uuid::new_v4();
let envelope = NewCommandEnvelope {
    command: NewCommand {
        command_type: COMMAND_TYPE.into(),
        payload: serde_json::to_string(&StartDailySourcingPayload {
            triggered_at: OffsetDateTime::now_utc(),
        })?,
    },
    metadata: Some(NewCommandMetadata {
        command_id,
        correlation_id: None,
        causation_id: None,
        source: Some("retailer-sourcing/daily-sourcing-endpoint".into()),
    }),
};
gateway.dispatch(envelope).map_err(internal_error)?;
```

`StartDailySourcingPayload` is a serde struct local to the feature module;
the command queue stores the JSON string verbatim.

### Wiring at startup

`bin/retailer-sourcing.rs` currently constructs `app()` with no
dependencies. It needs to:

1. Read `DATABASE_URL` and build a Diesel `Pool<ConnectionManager<PgConnection>>`.
2. Use `mulac_diesel` to build a `CommandStoreStorage` and from it a
   `CommandRecorder` (`commanding::io::CommandRecorder`).
3. Construct `CommandGateway::two_phased(Arc::new(recorder))`.
4. Read `HARDCODED_BEARER_TOKEN`.
5. Call `app(Arc::new(gateway), bearer_token)`.

The diesel pool, migrations, and `mulac_diesel` initialization are not
currently set up in this crate — they will need to be added as part of this
work. The exact diesel wiring (migration runner, schema module) follows
mulac's documented setup; see `~/.cargo/git/checkouts/mulac-*/libs/mulac_diesel`.

### Configuration

Add to environment (documented in `apps/retailer-sourcing/README.md`):

| Variable                 | Purpose                                |
|--------------------------|----------------------------------------|
| `DATABASE_URL`           | Postgres connection string for mulac   |
| `HARDCODED_BEARER_TOKEN` | Shared secret required on the endpoint |

Fail fast on startup if either is missing.

## File-by-file changes

- `apps/retailer-sourcing/Cargo.toml` — add features for `commanding` (with
  `diesel`), `mulac_diesel`, `diesel` (postgres + r2d2), `uuid` (v4 + serde),
  `time` (serde), `subtle`, `thiserror`, and existing `serde_json`.
- `apps/retailer-sourcing/src/bearer_auth.rs` — new `BearerAuth` middleware.
- `apps/retailer-sourcing/src/daily_sourcing.rs` — new feature module.
- `apps/retailer-sourcing/src/lib.rs` — `mod daily_sourcing;`, extend
  `pub mod io`, change `app()` signature to accept the gateway + token.
- `apps/retailer-sourcing/src/bin/retailer-sourcing.rs` — pool/recorder/
  gateway wiring; read env vars; pass into `app()`.
- `apps/retailer-sourcing/tests/daily_sourcing.rs` — integration tests
  (see verification).

## Verification

Integration tests in `tests/daily_sourcing.rs` using Poem's `TestClient`:

1. **Missing Authorization header → 401.**
2. **Wrong bearer token → 401.**
3. **Correct token → 202**, response body contains a UUID `command_id`, and
   a row exists in the commands table with `command_type = "StartDailySourcing"`
   and matching `command_id`.

For (3) the test sets up an in-memory or test-schema Postgres (via a fixture
helper to be added once mulac's recommended test harness is identified) and
asserts on the row through a `CommandStoreStorage` query.

### Bruno collection

A Bruno collection lives at `apps/retailer-sourcing/bruno/` and is the
canonical way to exercise the service against a running instance. Files:

- `bruno.json` — collection root.
- `environments/local.bru` — `host` and `bearer_token` for local dev.
- `check-health.bru` — sanity check on `/health`.
- `daily-sourcing.bru` — happy path, expects `202` + UUID `command_id`.
- `daily-sourcing-missing-auth.bru` — expects `401` when no auth header.
- `daily-sourcing-wrong-token.bru` — expects `401` on bad bearer token.

Each request file includes a `tests { ... }` block with assertions, so the
collection doubles as an end-to-end integration test runnable from CI via
the Bruno CLI:

```bash
cd apps/retailer-sourcing/bruno
bru run --env local
```

## Open questions

- Should the endpoint accept a JSON body (e.g. retailer filter, dry-run
  flag) or remain a pure trigger? Current plan assumes the latter.
- Token rotation: env var is sufficient for now; revisit when multiple
  callers need distinct credentials.
- Where do diesel migrations live in this repo? Needs a one-time decision
  before this work can ship.
