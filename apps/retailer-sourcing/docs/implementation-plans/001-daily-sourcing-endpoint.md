# 001 — Daily Sourcing Endpoint

## Context

Operators (or a scheduler) need a way to trigger the daily sourcing run on
demand. The retailer-sourcing service should expose a single HTTP endpoint
that, when called with a valid Bearer token, enqueues a `StartDailySourcing`
command. The command itself is not executed by the request — it is persisted
to the command queue (mulac's commanding store) and processed asynchronously
by a separate consumer. This separation keeps the HTTP request fast and
resilient to handler crashes while each request records a fresh command entry.

## Scope

- Add `POST /daily-sourcing/` to the existing Poem router.
- Reject requests without a valid `Authorization: Bearer <token>` header.
- Construct a `StartDailySourcing` command with a fresh `command_id` on each
  request and hand it to a `PersistentKernelState` configured with
  `drain_rounds = 0`, so the command is durably recorded in the
  `command_entries` table before the HTTP response returns, without executing
  any handlers in the request path.

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
  daily_sourcing.rs        // handler + register + io submodule
```

Public surface (re-exported through `lib.rs`'s top-level `pub mod io`):

- `pub const DAILY_SOURCING_PATH: &str = "/daily-sourcing/";`
- `pub const COMMAND_TYPE: &str = "StartDailySourcing";`

Private items:

- `#[handler] async fn trigger(...)` — receives an already-authenticated
  request, builds a `kernel::NewCommandEnvelope<StartDailySourcing>` with a
  freshly generated `command_id`, dispatches it through
  `PersistentKernelState::dispatch_command`, returns `202 Accepted` with the
  generated `command_id` on success. The handler is auth-agnostic: bearer
  validation is fully owned by the `BearerAuth` middleware and must not be
  duplicated here.
- `pub fn register(route: Route, state: Arc<PersistentKernelState>, bearer: Arc<String>) -> Route` —
  registers `trigger` at `io::DAILY_SOURCING_PATH`, attaches the kernel state
  via `AddData::new((*state).clone())`, and wraps the route entry with
  `BearerAuth::new((*bearer).clone())`. Each feature module registers itself on the
  shared `Route` passed in, avoiding double-nest conflicts.

> **Note:** `.with(...)` is a method on `poem::EndpointExt` — import it explicitly:
> `use poem::EndpointExt;`. `AddData::new(val: T)` stores `Arc<T>` internally;
> the handler extracts `Data<&T>` (not `Data<&Arc<T>>`), so pass the unwrapped
> value: `AddData::new((*state).clone())` where `state: Arc<PersistentKernelState>`.

Each feature module exposes a `register` function that takes and returns a
`Route`. `lib.rs::app()` becomes:

```rust
pub fn app(state: Arc<PersistentKernelState>, bearer: Arc<String>) -> Route {
    let route = Route::new();
    let route = check_health::register(route);
    daily_sourcing::register(route, state, bearer)
}
```

`check_health` gains a matching `pub fn register(route: Route) -> Route` that
does `.at(io::HEALTH_CHECK_PATH, get(health_check))`.

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

`BearerAuth` is attached inside `daily_sourcing::register` via
`.with(BearerAuth::new(bearer))`. The middleware module's `io` submodule
exposes only `BearerAuth`; the endpoint type stays private.

### Response schema

The endpoint returns `202 Accepted` with a JSON body:

```rust
#[derive(Serialize)]
pub struct DailySourcingResponse {
    pub command_id: Uuid,
}
```

Serialized JSON shape:

```json
{
  "command_id": "00000000-0000-0000-0000-000000000000"
}
```

The handler return type is `poem::Result<poem::Response>`. Return 202 using Poem's
tuple `IntoResponse` implementation:

```rust
use poem::IntoResponse;
Ok(
    (StatusCode::ACCEPTED, poem::web::Json(io::DailySourcingResponse { command_id }))
        .into_response(),
)
```

Export `DailySourcingResponse` in the feature `io` module so tests and Bruno
collection assertions can depend on it explicitly.

### Command envelope

`StartDailySourcing` is an application command implementing
`kernel::ApplicationCommand`. The kernel serializes it and routes it through
the commanding store.

```rust
#[derive(Serialize)]
struct StartDailySourcing {
    triggered_at: OffsetDateTime,
}

impl kernel::ApplicationCommand for StartDailySourcing {
    fn command_type(&self) -> &'static str {
        COMMAND_TYPE
    }
}
```

In the handler, build and dispatch the typed envelope:

```rust
let command_id = Uuid::now_v7();
let envelope = kernel::NewCommandEnvelope {
    command: StartDailySourcing {
        triggered_at: OffsetDateTime::now_utc(),
    },
    metadata: NewCommandMetadata {
        command_id,
        correlation_id: Some(command_id),
        causation_id: None,
        source: Some("retailer-sourcing.http".to_string()),
    },
};

// dispatch_command is synchronous — wrap in spawn_blocking so the async
// runtime thread is not blocked during the database write.
let state = state.0.clone(); // PersistentKernelState: Clone
tokio::task::spawn_blocking(move || state.dispatch_command(envelope))
    .await
    .map_err(|e| DailySourcingError::Enqueue(e.to_string()))? // JoinError
    .map_err(|e| DailySourcingError::Enqueue(e.to_string()))?; // KernelError

Ok(poem::web::Json(io::DailySourcingResponse { command_id }))
```

`state` is `Data<&PersistentKernelState>` extracted from the request by
the handler (`state.0.clone()` gives an owned `PersistentKernelState` that
can be moved into the blocking closure). The kernel converts the typed envelope
to a gateway envelope internally via `into_gateway_envelope`; no raw
`NewCommand`/`NewCommandEnvelope` is constructed in feature code.

### Error handling

Authentication failures return `401 Unauthorized` using Poem's
`Error::from_status(StatusCode::UNAUTHORIZED)`; no JSON error body is required
for auth failures.

Command dispatch failures return `500 Internal Server Error` with a minimal
JSON body:

```json
{
  "error": "command queue unavailable"
}
```

JSON serialization failures are treated as internal bugs and also return
`500 Internal Server Error`. UUID generation via `Uuid::now_v7()` is treated
as infallible for this endpoint.

Implement this with a small endpoint-local error type or helper that converts
serialization and command gateway errors into Poem responses. Avoid ad-hoc
placeholder error helpers without a documented response contract.

### Wiring at startup

`bin/retailer-sourcing.rs` should use the mulac kernel persistent startup
path. That startup path is responsible for:

1. Reading `DATABASE_URL` and building the Diesel pool via
   `mulac_diesel::build_pool`.
2. Building the kernel state with
   `kernel::boot(config).start_persistent(pool, 0)` — passing
   `drain_rounds = 0` ensures `dispatch_command` only records commands durably
   without executing handlers in the HTTP request path (enqueue-only
   semantics). The worker loop that executes commands is out of scope for
   this plan and runs separately.
3. Reading `RETAILER_SOURCING_BEARER_TOKEN`.
   // Single static shared-secret auth — adequate for internal-only use.
   // Revisit when more callers need distinct credentials (see open questions).
4. Passing `kernel_handle.state()` and the bearer token into `app()`.

The Diesel pool, migrations, and `mulac_diesel` initialization are not
currently set up in this crate — they will need to be added as part of this
work. The exact Diesel wiring (migration runner, schema module) follows
mulac's documented setup.

Reference points:

- [kernel/src/lib.rs](https://github.com/nulllabsdev/mulac/blob/main/kernel/src/lib.rs) for `boot`, `start_persistent`, `PersistentKernelState`, and `ApplicationCommand`.
- [test_apps/twitter/src/assembly/application.rs](https://github.com/nulllabsdev/mulac/blob/main/test_apps/twitter/src/assembly/application.rs)
  for a persistent kernel startup example.
- [test_apps/docker-compose.yml](https://github.com/nulllabsdev/mulac/blob/main/test_apps/docker-compose.yml) for the test
  service layout.

### Configuration

Add to environment (documented in `apps/retailer-sourcing/README.md`):

| Variable                         | Purpose                                |
|----------------------------------|----------------------------------------|
| `DATABASE_URL`                   | Postgres connection string for mulac   |
| `RETAILER_SOURCING_BEARER_TOKEN` | Shared secret required on the endpoint |

Fail fast on startup if either is missing.

## File-by-file changes

- `apps/retailer-sourcing/Cargo.toml` — replace speculative git deps with `kernel`
  (which re-exports commanding/eventing/etc.); add `uuid` (v4 + serde),
  `time` (serde-well-known), `subtle`, `thiserror`, `serde_json`, `diesel`
  (postgres + r2d2 + uuid), `diesel_migrations`, `dotenvy`; move `serde_json`
  from dev-dependencies to main.
- `apps/retailer-sourcing/migrations/YYYY-MM-DD-000001_infrastructure/up.sql` —
  copy the mulac infra tables SQL from
  `test_apps/twitter/migrations/…/infrastructure/up.sql` (creates
  `command_entries`, `event_entries`, `inbox_entries`, `outbox_entries`).
  Also add the matching `down.sql`.
- `apps/retailer-sourcing/src/bearer_auth.rs` — new `BearerAuth` middleware.
- `apps/retailer-sourcing/src/daily_sourcing.rs` — new feature module.
- `apps/retailer-sourcing/src/lib.rs` — `mod daily_sourcing; mod bearer_auth;`,
  extend `pub mod io` (re-export daily_sourcing io, expose `build_pool` and
  `run_migrations`), change `app()` signature to
  `app(state: Arc<PersistentKernelState>, bearer: Arc<String>) -> Route` using
  the `register` pattern; add `check_health::register`.
- `apps/retailer-sourcing/src/bin/retailer-sourcing.rs` — read `DATABASE_URL`
  and `RETAILER_SOURCING_BEARER_TOKEN`; call `run_migrations`; start kernel with
  `kernel::boot(config).start_persistent(pool, 0)`; pass
  `kernel_handle.state()` and bearer into `app()`.
- `apps/retailer-sourcing/tests/utils/mod.rs` — **new** shared test helpers:
  shared pool via `OnceLock`, `run_migrations`, `reset_tables` (TRUNCATE infra
  tables), `setup() -> (DbPool, Arc<PersistentKernelState>)`. Place in
  `tests/utils/mod.rs` (not `tests/utils.rs`) so Rust does not compile it as
  a standalone test binary.
- `apps/retailer-sourcing/tests/daily_sourcing.rs` — integration tests
  (see verification); add `mod utils;` at the top.
- `apps/retailer-sourcing/tests/check_health.rs` — update to call
  `app(state, Arc::new("unused".to_string()))` using `utils::setup()`, since
  `app()` now requires a kernel state and bearer token.
- `apps/retailer-sourcing/docker-compose.test.yml` — Postgres on port 5433
  (already described in Verification).

## Verification

### Test database setup

Tests run against a dedicated test Postgres instance managed by Docker Compose,
following the mulac pattern in
[test_apps/docker-compose.yml](https://github.com/nulllabsdev/mulac/blob/main/test_apps/docker-compose.yml).

Create `apps/retailer-sourcing/docker-compose.test.yml` with Postgres and
RabbitMQ, matching the services needed by the mulac kernel. Tests connect via
`DATABASE_URL` pointing to the test Postgres instance.

The integration test fixture owns database isolation:

- start or require the Docker Compose test services before the test run;
- apply the mulac/kernel migrations before assertions;
- truncate the `command_entries` table before each test case;
- expose helpers for querying persisted command rows by `command_id`.

### Integration tests

Integration tests in `tests/daily_sourcing.rs` using Poem's `TestClient`:

1. **Missing Authorization header → 401.**
   - Assert response status is 401 Unauthorized.

2. **Wrong bearer token → 401.**
   - Assert response status is 401 Unauthorized.

3. **Correct token → 202 Accepted with valid response schema.**
   - Assert response status is 202 Accepted.
   - Assert response body deserializes to `DailySourcingResponse`.
   - Assert `command_id` in the response is a valid UUID.
   - Assert a row exists in the `command_entries` table with `command_type = "StartDailySourcing"`
     and `command_id` matching the response body.

4. **Enqueue-only: handler does not execute the command.**
   - Assert that after the 202 response, no `StartDailySourcing` handler has
     run (verified by confirming the command row is in `Received` status, not
     `Processed`).

For (3) and (4), the test fixture sets up the test Postgres connection,
truncates the `command_entries` table, and queries via `CommandStoreStorage`
to verify persistence.

### Bruno collection

A Bruno collection lives at `apps/retailer-sourcing/bruno/` and is the
canonical way to exercise the service against a running instance. Files:

- `bruno.json` — collection root.
- `environments/local.bru` — `host` and `bearer_token` for local dev.
- `check-health.bru` — sanity check on `/health`.
- `daily-sourcing.bru` — happy path, expects `202 Accepted` with JSON response
  matching `DailySourcingResponse { command_id: Uuid }`.
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

- **S001 (Deferred):** Bootstrap ownership model across binary, app(), and mulac kernel startup path remains ambiguous. This requires team alignment on responsibility boundaries before implementation.
- Token rotation: env var (`RETAILER_SOURCING_BEARER_TOKEN`) is sufficient for now; revisit when multiple callers need distinct credentials.
