# S017 - Wrap dispatch_command in spawn_blocking

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | high                                                                              |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | commit 013880c                                                                    |
| Created at               | 2026-05-25                                                                        |
| Author                   | GitHub Copilot, claude-sonnet-4.6, high                                           |
| Reviewer                 | GitHub Copilot, claude-sonnet-4.6                                                 |

## Issue

The handler code sketch in the plan calls `state.dispatch_command(envelope)` directly
inside an async handler:

```rust
state.dispatch_command(envelope).map_err(DailySourcingError::from)?;
```

`PersistentKernelState::dispatch_command` is a **synchronous, blocking function** that
hits the database. Calling it directly on the Tokio async executor will stall the
runtime thread for the duration of the DB write, degrading throughput under any load.
This is a silent correctness hazard — it compiles and passes tests on a lightly loaded
machine, then degrades silently in production.

## Suggestion

Update the handler code sketch to wrap the dispatch call in
`tokio::task::spawn_blocking`:

```rust
let state = state.0.clone(); // PersistentKernelState: Clone
tokio::task::spawn_blocking(move || state.dispatch_command(envelope))
    .await
    .map_err(|e| DailySourcingError::Enqueue(e.to_string()))? // JoinError
    .map_err(|e| DailySourcingError::Enqueue(e.to_string()))?; // KernelError
```

Add a prose note: "`dispatch_command` is synchronous. Wrap it in
`tokio::task::spawn_blocking` so the async runtime thread is not blocked during
the database write."
