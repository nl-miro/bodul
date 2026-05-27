# 002 — Daily Sourcing Flow

## Context

[Plan 001](001-daily-sourcing-endpoint.md) added the `POST /daily-sourcing/`
HTTP endpoint that persists a `StartDailySourcing` command. That command is
currently unhandled — no event is emitted and no downstream work happens.

This plan covers the end-to-end **daily sourcing flow**: from the
`StartDailySourcing` command being picked up by a handler, through an event
that announces "today's sourcing has been requested for these retailers", to
fan-out commands that schedule one sourcing job per (retailer, method)
combination.

The actual scraping/parsing logic that consumes `SourceRetailer` commands is
**not** part of this plan — each method handler will land in its own
follow-up plan. This plan stops at "the `SourceRetailer` commands are
persisted in `command_entries`, ready to be picked up by their handlers".

## Scope

### In scope

- Define `DailySourcingHandler` that handles `StartDailySourcing` and emits
  a `DailySourcingRequested` event carrying the list of active retailer codes.
- Define `DailySourcingRequested` application event.
- Define an `ActiveRetailers` specification module that returns the set of
  retailer codes currently considered active. For now, it returns every
  variant of `retailer_guild::io::RetailerCode`.
- Define `SourceRetailer` command (`retailer_code`, `method`) and the
  `SourcingMethod` enum (`WebScraping`, `Sitemap`).
- Define `DailySourcingFanOutSubscriber` that subscribes to
  `DailySourcingRequested` and dispatches one `SourceRetailer` command per
  `(retailer_code, method)` tuple.
- Introduce an app-level event enum `RetailerSourcingEvent` (mirroring
  mulac's `TwitterEvent` pattern) so handlers can return typed events through
  `CommandHandlerPort<C, RetailerSourcingEvent>`.
- Register the new handler, event, and subscriber in the kernel boot path.
- Start command + event worker loops in the service binary so commands and
  events actually get processed (today the binary only serves HTTP and
  drains nothing).

### Out of scope

- Handlers for `SourceRetailer` (one plan per method — web scraping in plan
  003, sitemap in plan 004).
- The retailer-specific scraping, parsing, or downstream emission of any
  domain events from those handlers.
- Dynamic active-retailer configuration (database table, admin endpoint,
  retailer-management lookup). Tracked as an open question.
- Idempotency / dedup of `SourceRetailer` commands across runs (see
  [S011](../suggestions/001-daily-sourcing-endpoint/S011_idempotency-and-retries_claude-opus-4-7-medium.md),
  deferred).

## Design

### Module layout

Following [coding-guidelines.md](../../../../docs/coding-guidelines.md) (one
file per feature, private to `lib.rs`, public surface only through `io`):

```text
src/
  daily_sourcing.rs      // existing: HTTP endpoint + StartDailySourcing
                         // command. Extended with DailySourcingHandler and
                         // the DailySourcingRequested event.
  source_retailer.rs     // NEW: SourceRetailer command + SourcingMethod enum
                         // + DailySourcingFanOutSubscriber that turns
                         // DailySourcingRequested into N*M SourceRetailer
                         // commands.
  active_retailers.rs    // NEW: ActiveRetailers specification — returns the
                         // list of retailer codes the service considers
                         // active. For now, all RetailerCode variants.
  app_event.rs           // NEW: RetailerSourcingEvent enum that aggregates
                         // every event emitted by handlers in this app.
                         // Implements kernel::ApplicationEvent.
  assembly.rs            // NEW: start_mulac(pool) — single place where every
                         // command handler and event subscriber is wired
                         // into the kernel. Mirrors mulac's twitter test
                         // app pattern.
```

`lib.rs` declares them as private modules and re-exports each feature's `io`
through the top-level `pub mod io`.

### Application event enum

Like mulac's `TwitterEvent`, every handler returns
`Vec<RetailerSourcingEvent>` from its `CommandHandlerPort` impl. The
top-level enum lets the kernel route emitted events to subscribers by
`event_type`.

```rust
// src/app_event.rs
use serde::{Deserialize, Serialize};

pub mod io {
    pub use super::RetailerSourcingEvent;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RetailerSourcingEvent {
    DailySourcingRequested(crate::daily_sourcing::io::DailySourcingRequested),
}

impl kernel::ApplicationEvent for RetailerSourcingEvent {
    fn event_type(&self) -> &'static str {
        match self {
            Self::DailySourcingRequested(e) => e.event_type(),
        }
    }
}
```

`RetailerSourcingEvent` is re-exported via the feature's `pub mod io` (and
in turn through `lib.rs::io`). The handler imports it via
`crate::app_event::io::RetailerSourcingEvent`, never reaching into the
module root directly. Integration tests use `crate::io::RetailerSourcingEvent`.

New events added by future plans (e.g. `RetailerSourced`, `SourcingFailed`)
extend this enum.

### Active retailers specification

```rust
// src/active_retailers.rs
use retailer_guild::io::RetailerCode;

pub mod io {
    pub use super::ActiveRetailers;
}

pub struct ActiveRetailers;

impl ActiveRetailers {
    pub fn list() -> Vec<RetailerCode> {
        vec![
            RetailerCode::MINISFORUM_EU,
            RetailerCode::MINISFORUM_US,
            RetailerCode::MINISFORUM_UK,
            RetailerCode::MINISFORUM_FR,
            RetailerCode::MINISFORUM_CA,
            RetailerCode::MINISFORUM_AU,
        ]
    }
}
```

This is a deliberate single point of change: when "active" gets a real
definition (DB-backed, admin-toggleable, etc.), every caller already goes
through `ActiveRetailers::list()`.

`RetailerCode` does not currently derive `Clone`/`Debug`/`Serialize` — the
spec's consumers need at least `Clone` (to copy into command payloads) and
`Serialize` (so it can be persisted in `SourceRetailer.payload`). Add those
derives in `lib/retailer-guild/src/lib.rs` as part of this plan.

### `daily_sourcing.rs` additions

The file already has `StartDailySourcing` (command), `trigger` (HTTP
handler), and `register` (route registration). Extend it with:

```rust
// in src/daily_sourcing.rs

// existing application submod gains:

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySourcingRequested {
    pub retailer_codes: Vec<RetailerCode>,
    pub requested_at: DateTime<Utc>,
}

impl kernel::ApplicationEvent for DailySourcingRequested {
    fn event_type(&self) -> &'static str {
        EVENT_TYPE_DAILY_SOURCING_REQUESTED
    }
}

pub const EVENT_TYPE_DAILY_SOURCING_REQUESTED: &str = "DailySourcingRequested";
```

And a new `handler` submod:

```rust
mod handler {
    use super::application::{DailySourcingRequested, StartDailySourcing};
    use crate::active_retailers::io::ActiveRetailers;
    use crate::app_event::io::RetailerSourcingEvent;
    use chrono::Utc;
    use kernel::io::{CommandError, CommandHandlerPort};

    pub struct DailySourcingHandler;

    impl DailySourcingHandler {
        pub fn new() -> Self {
            Self
        }
    }

    impl CommandHandlerPort<StartDailySourcing, RetailerSourcingEvent>
        for DailySourcingHandler
    {
        fn execute(
            &self,
            _cmd: StartDailySourcing,
        ) -> Result<Vec<RetailerSourcingEvent>, CommandError> {
            let event = DailySourcingRequested {
                retailer_codes: ActiveRetailers::list(),
                requested_at: Utc::now(),
            };
            Ok(vec![RetailerSourcingEvent::DailySourcingRequested(event)])
        }
    }
}
```

The handler is intentionally trivial — it owns one responsibility: "snapshot
which retailers are active right now, announce that we want to source them".
Fan-out work happens in the subscriber, not here.

The feature's `pub mod io` is extended:

```rust
pub mod io {
    pub use super::application::{
        DailySourcingRequested,
        StartDailySourcing,
        EVENT_TYPE_DAILY_SOURCING_REQUESTED,
    };
    pub use super::handler::DailySourcingHandler;
    pub use super::http::{DAILY_SOURCING_PATH, DailySourcingResponse, register, trigger};
}
```

### `source_retailer.rs`

```rust
// src/source_retailer.rs

const COMMAND_TYPE: &str = "SourceRetailer";

pub mod io {
    pub use super::application::{SourceRetailer, SourcingMethod};
    pub use super::eventing::DailySourcingFanOutSubscriber;
    pub use super::COMMAND_TYPE;
}

mod application {
    use chrono::{DateTime, Utc};
    use retailer_guild::io::RetailerCode;
    use serde::{Deserialize, Serialize};

    // `rename_all = "PascalCase"` is a no-op for the current variants but
    // pins the JSON wire format ("WebScraping"/"Sitemap") even if a new
    // variant is added with non-PascalCase casing. Renaming a variant is a
    // wire-format breaking change since payloads are stored verbatim in
    // SourceRetailer.payload.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    #[serde(rename_all = "PascalCase")]
    pub enum SourcingMethod {
        WebScraping,
        Sitemap,
    }

    impl SourcingMethod {
        pub fn all() -> &'static [SourcingMethod] {
            &[SourcingMethod::WebScraping, SourcingMethod::Sitemap]
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SourceRetailer {
        pub retailer_code: RetailerCode,
        pub method: SourcingMethod,
        pub requested_at: DateTime<Utc>,
    }

    impl kernel::ApplicationCommand for SourceRetailer {
        fn command_type(&self) -> &'static str {
            super::COMMAND_TYPE
        }
    }
}

mod eventing {
    use super::application::{SourceRetailer, SourcingMethod};
    use crate::daily_sourcing::io::DailySourcingRequested;
    use kernel::io::{
        CommandGateway,
        NewCommand,
        NewCommandEnvelope,
        NewCommandMetadata,
    };
    use kernel::{EventError, EventSubscriberPort, NewEventEnvelope};
    use std::sync::Arc;
    use uuid::Uuid;

    pub struct DailySourcingFanOutSubscriber {
        command_gateway: Arc<CommandGateway>,
    }

    impl DailySourcingFanOutSubscriber {
        pub fn new(command_gateway: Arc<CommandGateway>) -> Self {
            Self { command_gateway }
        }
    }

    impl EventSubscriberPort for DailySourcingFanOutSubscriber {
        fn handle(
            &self,
            envelope: &NewEventEnvelope,
        ) -> Result<(), EventError> {
            // `NewEventEnvelope::metadata` is `Option<NewEventMetadata>`;
            // a properly-emitted DailySourcingRequested always carries it,
            // but we still unwrap explicitly rather than panicking.
            let meta = envelope.metadata.as_ref().ok_or_else(|| {
                EventError::SubscriberExecution(
                    "DailySourcingRequested event is missing metadata"
                        .to_string(),
                )
            })?;
            let event: DailySourcingRequested =
                serde_json::from_str(&envelope.payload)
                    .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;

            for retailer_code in &event.retailer_codes {
                for method in SourcingMethod::all() {
                    let cmd = SourceRetailer {
                        retailer_code: retailer_code.clone(),
                        method: *method,
                        requested_at: event.requested_at,
                    };
                    let gateway_envelope = NewCommandEnvelope {
                        command: NewCommand {
                            command_type: super::COMMAND_TYPE.to_string(),
                            payload: serde_json::to_string(&cmd).map_err(
                                |e| EventError::SubscriberExecution(e.to_string()),
                            )?,
                        },
                        metadata: Some(NewCommandMetadata {
                            command_id: Uuid::now_v7(),
                            // Propagate the saga's correlation_id (set by
                            // the HTTP handler on StartDailySourcing) so
                            // every SourceRetailer command shares it.
                            correlation_id: meta.correlation_id,
                            // The DailySourcingRequested event is the
                            // immediate cause of this SourceRetailer.
                            causation_id: Some(meta.event_id),
                            source: Some(
                                "event:DailySourcingRequested".to_string(),
                            ),
                        }),
                    };
                    self.command_gateway
                        .dispatch(gateway_envelope)
                        .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
                }
            }
            Ok(())
        }
    }
}
```

The subscriber uses the lower-level `CommandGateway::dispatch` directly
(rather than `PersistentKernelState::dispatch_command`) because that is what
the mulac kernel hands subscribers via `event_subscriber_with_command_gateway`.
The gateway is configured with `two_phased` in the persistent kernel, so
dispatch is enqueue-only — the new `SourceRetailer` commands land in
`command_entries` with status `Received` and are picked up by the worker loop
later.

### Kernel boot wiring

Following mulac's twitter test app
([`assembly/application.rs::start_mulac`](https://github.com/nulllabsdev/mulac/blob/main/test_apps/twitter/src/assembly/application.rs)),
**all kernel registrations live in a single function** in a new
`src/assembly.rs` module. The binary stays thin per the coding guidelines:
read env vars, build the pool, run migrations, call `start_mulac(pool)`,
spawn workers, run the HTTP server.

```rust
// src/assembly.rs

pub mod io {
    pub use super::start_mulac;
    pub use kernel::io::{run_command_worker, run_event_worker};
}

use crate::daily_sourcing;
use crate::source_retailer;
use kernel::io::DbPool;
use kernel::{KernelConfig, KernelError, PersistentKernelHandle, boot};
use std::sync::Arc;

pub fn start_mulac(
    pool: DbPool,
) -> Result<PersistentKernelHandle, KernelError> {
    boot(KernelConfig::default())
        .command_handler(
            daily_sourcing::io::COMMAND_TYPE,
            Arc::new(daily_sourcing::io::DailySourcingHandler::new()),
        )
        .event_subscriber_with_command_gateway(
            daily_sourcing::io::EVENT_TYPE_DAILY_SOURCING_REQUESTED,
            "daily-sourcing-fan-out",
            |command_gateway| {
                Arc::new(
                    source_retailer::io::DailySourcingFanOutSubscriber::new(
                        command_gateway,
                    ),
                ) as Arc<dyn kernel::EventSubscriberPort>
            },
        )
        .start_persistent(pool, 0)
}
```

`drain_rounds = 0` is preserved — HTTP dispatch must stay enqueue-only. The
actual draining happens in the worker loops described next.

`COMMAND_TYPE` for `StartDailySourcing` is currently a private const in
`daily_sourcing.rs`; expose it through `io` so `assembly.rs` can pass it to
`command_handler(...)` without string duplication.

The integration test's `setup_full_kernel` helper calls the same
`start_mulac(pool)` so the test kernel matches the production kernel
exactly — no risk of test/prod registration drift.

### Worker loops in the binary

Today, the binary serves HTTP and exits when killed. Commands and events are
never drained from their tables — `StartDailySourcing` rows would just
accumulate. The binary calls `start_mulac(pool)` (from `assembly.rs`), then
spawns the workers:

```rust
let handle = retailer_sourcing::io::start_mulac(pool)?;

let token = handle.child_token();
let command_worker = tokio::spawn(retailer_sourcing::io::run_command_worker(
    handle.command_consumer(),
    token.clone(),
));
let event_worker = tokio::spawn(retailer_sourcing::io::run_event_worker(
    handle.event_consumer(),
    token.clone(),
));

let state = Arc::new(handle.state());
let bearer = Arc::new(bearer_token);

// Monitor every long-running task. If any of them exits — HTTP server,
// command worker, or event worker — trigger a graceful shutdown of the
// rest. Workers that exit silently while the HTTP server keeps returning
// 202 would otherwise produce a hard-to-detect partial outage where
// commands and events stop being processed.
tokio::select! {
    res = Server::new(TcpListener::bind(BIND_ADDRESS)).run(app(state, bearer)) => {
        if let Err(e) = res {
            eprintln!("HTTP server error: {e}");
        }
    }
    res = command_worker => {
        eprintln!("Command worker exited unexpectedly: {res:?}");
    }
    res = event_worker => {
        eprintln!("Event worker exited unexpectedly: {res:?}");
    }
    _ = tokio::signal::ctrl_c() => {
        println!("Received SIGINT, shutting down...");
    }
}
handle.shutdown();
handle.wait().await?;
```

This co-locates the HTTP server and the workers in the same binary, which
is the simplest setup. The `tokio::select!` block treats every long-running
task as a shutdown signal so any silent worker death takes the whole
process down (and lets the orchestrator restart it) rather than leaving an
HTTP server happily acknowledging requests that nothing will ever process.
A separate worker binary can be split out later if operational needs
diverge.

### Cardinality and ordering

Given today's `ActiveRetailers::list()` (6 MinisForum codes) and 2 methods,
one `StartDailySourcing` produces:

- 1 `DailySourcingRequested` event,
- 12 `SourceRetailer` commands.

No ordering is guaranteed between the `SourceRetailer` commands — the fan-out
subscriber dispatches them sequentially, but the worker loop reserves and
processes them independently. Handlers for `SourceRetailer` must not assume
ordering.

### Error handling

- `DailySourcingHandler::execute` returns `Result<_, CommandError>`. The
  handler currently has no fallible operation; it always succeeds.
- `DailySourcingFanOutSubscriber::handle` returns `Result<_, EventError>`.
  Failures from `serde_json` or `command_gateway.dispatch` are mapped to
  `EventError::SubscriberExecution`. The kernel's event consumer is
  responsible for retry/poison-message handling per its existing policy; we
  do not introduce per-feature retry logic here.
- **Partial fan-out failure.** The subscriber dispatches N×M commands
  sequentially. If a dispatch fails mid-loop (e.g. transient DB error on
  dispatch #7), the subscriber returns `EventError::SubscriberExecution`
  and the kernel retries the event. The already-persisted commands from
  the failed attempt remain in `command_entries` with their original
  random UUIDs; the retry produces a fresh set of N×M commands with new
  UUIDs. Until fan-out idempotency lands (deterministic command_ids — see
  the open question below, deferred from S008), `SourceRetailer` handlers
  must be idempotent themselves and the operator should expect duplicate
  `SourceRetailer` rows on transient infra errors.

## File-by-file changes

- `apps/retailer-sourcing/Cargo.toml` — add path dep on `retailer-guild`
  (`path = "../../lib/retailer-guild"`); add `serial_test = "3"` to
  `[dev-dependencies]`.
- `apps/retailer-sourcing/src/lib.rs` —
  `mod active_retailers; mod app_event; mod assembly; mod source_retailer;`,
  re-export each feature's `io` (including `assembly::io` so the binary
  imports `start_mulac` / `run_command_worker` / `run_event_worker`
  through `retailer_sourcing::io`).
- `apps/retailer-sourcing/src/daily_sourcing.rs` —
  - Add `application::DailySourcingRequested` + its
    `kernel::ApplicationEvent` impl.
  - Add `handler` submod with `DailySourcingHandler`.
  - Re-export the new items through this feature's `io`.
  - Expose `COMMAND_TYPE` through `io`.
- `apps/retailer-sourcing/src/active_retailers.rs` — **new** module.
- `apps/retailer-sourcing/src/app_event.rs` — **new** module with
  `RetailerSourcingEvent` enum.
- `apps/retailer-sourcing/src/source_retailer.rs` — **new** module with
  `SourceRetailer`, `SourcingMethod`, `DailySourcingFanOutSubscriber`.
- `apps/retailer-sourcing/src/assembly.rs` — **new** module exposing
  `pub fn start_mulac(pool) -> Result<PersistentKernelHandle, KernelError>`
  that registers every command handler and event subscriber in one place.
  Re-exports `kernel::io::{run_command_worker, run_event_worker}` for the
  binary's convenience.
- `apps/retailer-sourcing/src/bin/retailer-sourcing.rs` —
  - Call `retailer_sourcing::io::start_mulac(pool)` (no inline kernel
    registration).
  - Spawn `run_command_worker` and `run_event_worker`, keeping their
    `JoinHandle`s.
  - Wrap the HTTP server in `tokio::select!` with `ctrl_c` and worker
    `JoinHandle`s for graceful shutdown.
- `lib/retailer-guild/src/lib.rs` — add `serde` to dependencies; derive
  `Clone, Debug, PartialEq, Eq, Serialize, Deserialize` on `RetailerCode`
  and route serde through the existing PascalCase string representation:

  ```rust
  #[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(into = "String", try_from = "String")]
  pub enum RetailerCode { … }
  ```

  Replace the panicking `From<String>` with a `TryFrom<String>` returning a
  `Result<Self, String>` (use the same variant table as today's
  `as_string()`), and add a `From<RetailerCode> for String` that delegates
  to `as_string().to_string()`. This guarantees JSON payloads carry
  `"MinisForumEU"`/`"MinisForumUS"`/etc. — matching `as_string()` — instead
  of the default-derived `"MINISFORUM_EU"`, which would otherwise diverge
  between in-memory enums and persisted command payloads.
- `apps/retailer-sourcing/tests/daily_sourcing_flow.rs` — **new**
  integration test (see verification). Imports `serial_test::serial` and
  marks every test with `#[serial]`.
- `apps/retailer-sourcing/tests/utils/mod.rs` — add `wait_for` polling
  helper (see verification).
- `apps/retailer-sourcing/Makefile` — add `run` target that invokes
  `cargo run --bin retailer-sourcing` (sourcing env vars from `.env` via
  `dotenvy`, already wired in the binary).

## Verification

### Unit-style assertions on `ActiveRetailers`

A simple test in `active_retailers.rs` or a sibling test file asserts that
`ActiveRetailers::list()` returns exactly the 6 known MinisForum codes. If
that list changes, this test fails and forces the change to be deliberate.

### Integration test: `tests/daily_sourcing_flow.rs`

The existing test harness in `tests/utils/mod.rs` already builds a kernel
state with `drain_rounds = 0`. For this flow we need the **full** kernel
(handlers + subscribers registered) plus the worker loops, so the test sets
up its own kernel via a helper:

```rust
fn setup_full_kernel() -> (DbPool, Arc<PersistentKernelState>, KernelHandle);
```

This helper:

1. truncates the infra tables (existing `reset_tables`);
2. calls `retailer_sourcing::io::start_mulac(pool)` — the same function the
   binary uses, so there is no risk of test/prod kernel drift;
3. spawns `run_command_worker` and `run_event_worker` for the duration of
   the test (use a `CancellationToken` and shut them down at end-of-test);
4. returns the pool, state, and handle.

#### Polling helper

The kernel workers poll their tables every 1 s (`POLL_INTERVAL`). Tests
need a small helper that waits for the worker to make progress without
hard-coding sleeps. Add to `tests/utils/mod.rs`:

```rust
use std::time::Duration;

/// Poll `predicate` at 100 ms intervals until it returns `true` or
/// `timeout` elapses. Panics on timeout.
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
```

#### Status values

`command_entries.status` and `event_entries.status` are persisted from
mulac's `CommandStatus` / `EventStatus` enums. The integer encoding used
by the diesel layer:

| Value | Meaning   |
|-------|-----------|
| 0     | Received  |
| 1     | Reserved  |
| 2     | Processed |

Source of truth:
[libs/commanding/src/assembly](https://github.com/nulllabsdev/mulac/tree/main/libs/commanding/src/assembly)
(and the parallel `eventing` directory). Use named constants in tests
rather than literals so the mapping is grep-able:

```rust
const STATUS_RECEIVED: i32 = 0;
const STATUS_PROCESSED: i32 = 2;
```

Use a 10 s timeout in tests (10× the worker poll interval — plenty of
headroom without slowing CI):

```rust
wait_for(Duration::from_secs(10), || {
    let status: i32 = /* query command_entries.status WHERE id = … */;
    status == STATUS_PROCESSED
})
.await;
```

Tests, marked `#[serial]` (DB is shared):

1. **`StartDailySourcing` is consumed and emits `DailySourcingRequested`.**
   - Dispatch a `StartDailySourcing` via the kernel state.
   - Use `wait_for` to poll `command_entries` until the row's `status`
     reaches `Processed`.
   - Assert exactly one row in `event_entries` with
     `event_type = "DailySourcingRequested"`.
   - Assert the payload deserializes to `DailySourcingRequested` with
     `retailer_codes == ActiveRetailers::list()`.

2. **Fan-out produces N×M `SourceRetailer` commands.**
   - After the same dispatch, poll until `event_entries` row reaches
     `Processed` (subscriber has run).
   - Assert exactly
     `ActiveRetailers::list().len() * SourcingMethod::all().len()` rows in
     `command_entries` with `command_type = "SourceRetailer"`.
   - For every `(retailer_code, method)` combination from
     `ActiveRetailers::list() × SourcingMethod::all()`, assert at least one
     row exists with a matching payload.

3. **Each `SourceRetailer` command carries the correct causation/correlation
   chain.**
   - Query the `DailySourcingRequested` row's `event_id` and the originating
     `StartDailySourcing` row's `correlation_id` (set by the HTTP handler).
   - Assert every `SourceRetailer` row's `causation_id` equals the event_id
     of `DailySourcingRequested` (immediate cause).
   - Assert every `SourceRetailer` row's `correlation_id` equals the
     `correlation_id` of the originating `StartDailySourcing` — confirms the
     saga thread propagates end-to-end across HTTP → command → event →
     fan-out commands.

`SourceRetailer` handlers are deliberately not registered in the test
kernel — the commands should sit in `command_entries` with `status =
Received` after fan-out completes. Asserting this status is the explicit
"enqueue-only" check for fan-out output and parallels the assertion already
made for `StartDailySourcing` in plan 001.

### Bruno collection

No new Bruno requests for this plan — the existing
`daily-sourcing.bru` triggers the full flow end-to-end once the worker loop
is running in the binary. Manual smoke test:

```bash
make up && make run    # one terminal
cd apps/retailer-sourcing/bruno && bru run --env local
# in psql:
SELECT count(*) FROM command_entries WHERE command_type = 'SourceRetailer';
# expect 12
```

## Open questions

- **Active retailer source of truth.** Hardcoded today.
  Future options:
  (a) `retailers` table in this service's DB with an `active` boolean;
  (b) call out to `retailer-management` over HTTP/gRPC;
  (c) configuration file + reload on SIGHUP.
  Decision needed before either retailer onboarding becomes operator-driven
  or a second retailer family (non-MinisForum) is added.

- **Fan-out idempotency.** If the worker reprocesses a
  `DailySourcingRequested` event (e.g. after a crash between subscriber
  ack and event marking), the subscriber will dispatch a second set of
  `SourceRetailer` commands. Deferred per
  [S011](../suggestions/001-daily-sourcing-endpoint/S011_idempotency-and-retries_claude-opus-4-7-medium.md);
  a deterministic command_id derived from
  `(event_id, retailer_code, method)` would make `SourceRetailer` rows
  unique and the fan-out idempotent. Revisit before production.

- **Worker co-location.** This plan runs HTTP and workers in one binary
  for operational simplicity. Splitting into a separate `retailer-sourcing-worker`
  binary may become necessary if HTTP latency starts being affected by
  heavy scraping handlers, or if the worker needs separate scaling.
