# S004 — Specify concrete polling strategy for integration tests

| Field                    | Value                                                                                                                                                                                                                                              |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                             |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                      |
| Decision                 | accepted                                                                                                                                                                                                                                           |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Verification section adds "Polling helper" subsection with `wait_for` implementation and 10 s timeout convention; file-by-file lists `tests/utils/mod.rs` addition) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                         |
| Author                   | Claude Code, claude-sonnet-4-6, medium                                                                                                                                                                                                             |
| Reviewer                 |                                                                                                                                                                                                                                                    |

## Issue

The verification section says to "poll `command_entries` until the row's
`status` reaches `Processed` with a sensible timeout", but gives no concrete
implementation guidance. During implementation this ambiguity leads to
inconsistent polling helpers across tests, or slow tests from overly long
sleeps.

The kernel workers poll every 1 s (`POLL_INTERVAL`). Without knowing this, an
implementer might poll every 100 ms (creating unnecessary DB load) or every
5 s (making tests slow).

## Suggestion

Add a `wait_for` polling helper to `tests/utils/mod.rs` and document it in
the plan:

```rust
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

Use a 10 s timeout in tests (10× the worker poll interval, plenty of headroom
without making CI slow):

```rust
wait_for(Duration::from_secs(10), || {
    let status = /* query command_entries.status WHERE id = … */;
    status == 2 // Processed
})
.await;
```

Add this helper to the file-by-file section:
`apps/retailer-sourcing/tests/utils/mod.rs` — add `wait_for` helper.
