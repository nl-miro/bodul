# S009 — Kernel boot wiring location is left as "bin or assembly module"

| Field                    | Value                                                                                                                                                                                                                                                                                                                                            |
|--------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                                                                                                                                             |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                                                                                                    |
| Decision                 | accepted                                                                                                                                                                                                                                                                                                                                         |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Added `src/assembly.rs` with `start_mulac` to module layout, kernel boot wiring, and file-by-file sections; binary entry now calls `retailer_sourcing::io::start_mulac(pool)`; integration-test helper calls the same `start_mulac` to eliminate test/prod drift) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                                                                                       |
| Author                   | Claude Code, claude-sonnet-4-6, high                                                                                                                                                                                                                                                                                                             |
| Reviewer                 |                                                                                                                                                                                                                                                                                                                                                  |

## Issue

The "Kernel boot wiring" section opens with:

> `bin/retailer-sourcing.rs` (or a small `kernel_assembly` module) needs:

… followed by a substantial code block that registers the handler and the
fan-out subscriber, then starts the persistent kernel. The "Worker loops in
the binary" section also lives in the binary.

Two problems compound here:

1. **Coding guidelines say the binary is a thin wrapper around `lib::app()`**
   (see `docs/coding-guidelines.md`). Registering command handlers and event
   subscribers in the binary breaks that contract, and every future feature
   that adds a handler/subscriber would need to extend `main.rs` instead of
   composing through `lib.rs`.
2. **"or a small `kernel_assembly` module" is hand-wavy.** No file path, no
   API surface, no relationship to `lib::app()`. Two implementers will land
   in two different places: one will pile registrations into
   `bin/retailer-sourcing.rs`, the other will invent `src/kernel.rs` or
   `src/assembly.rs` from scratch.

The mulac twitter test app has a clear pattern:
`test_apps/twitter/src/assembly/application.rs::start_mulac(pool) ->
PersistentKernelHandle` is the single place where every handler and
subscriber is registered. The binary calls `start_mulac(pool)`, spawns the
workers, and runs the HTTP server.

## Suggestion

Make the location concrete and align with the twitter pattern.

1. Add a new private module `apps/retailer-sourcing/src/assembly.rs` (or
   rename `app_event.rs` to `assembly.rs` and host both there) that exposes
   a single function:

   ```rust
   pub fn start_mulac(
       pool: DbPool,
   ) -> Result<PersistentKernelHandle, KernelError> {
       kernel::boot(KernelConfig::default())
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

2. Re-export `start_mulac` (and `run_command_worker`, `run_event_worker`
   convenience re-exports) through the crate's top-level `pub mod io`, so
   the binary imports `retailer_sourcing::io::{start_mulac,
   run_command_worker, run_event_worker}`.

3. Update the "Kernel boot wiring" section to point at `assembly.rs`, drop
   the "or a small … module" alternative, and update file-by-file to list
   `src/assembly.rs` as a new module.

4. Update the binary to call `start_mulac(pool)` rather than build the
   kernel inline. The binary stays thin: read env vars, build pool, run
   migrations, `start_mulac`, spawn workers, run server, wait for shutdown.

This keeps the contract from the coding guidelines and makes the
integration test's "same handler/subscriber registrations as the binary"
helper trivial — it just calls `start_mulac(pool)`.
