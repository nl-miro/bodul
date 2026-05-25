# S011 — Partial fan-out failure leaves partial state; error handling section omits it

| Field                    | Value                                                                                                                                                                                                                                                                              |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                                             |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                                      |
| Decision                 | accepted                                                                                                                                                                                                                                                                           |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Error handling section now includes a "Partial fan-out failure" bullet describing partial-state risk on mid-loop dispatch errors and tying the operational mitigation to deferred idempotency work) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                         |
| Author                   | Claude Code, claude-sonnet-4-6, medium                                                                                                                                                                                                                                             |
| Reviewer                 |                                                                                                                                                                                                                                                                                    |

## Issue

The fan-out subscriber dispatches the N×M commands sequentially with
`?`-style early return:

```rust
for retailer_code in &event.retailer_codes {
    for method in SourcingMethod::all() {
        …
        self.command_gateway
            .dispatch(gateway_envelope)
            .map_err(|e| EventError::SubscriberExecution(e.to_string()))?;
    }
}
```

If dispatch #7 of 12 fails (DB connection lost mid-loop, transient write
error), the subscriber returns `Err`, the kernel marks the event for retry,
and **commands #1–6 are already persisted** with random `Uuid::now_v7()`
IDs. On the next retry, the loop starts again at #1 with **fresh random
UUIDs**, producing duplicate rows for retailers/methods 1–6.

This is the same root cause that S008 (deferred) raised, but the *partial*
case is worse than the *full retry* case: a flaky DB during fan-out produces
non-deterministic duplication patterns, and the integration test "exactly
12 `SourceRetailer` rows" would pass on a healthy CI database while silent
duplication piles up in production.

The plan's "Error handling" section currently only says:

> Failures from `serde_json` or `command_gateway.dispatch` are mapped to
> `EventError::SubscriberExecution`. The kernel's event consumer is
> responsible for retry/poison-message handling per its existing policy;
> we do not introduce per-feature retry logic here.

This is technically correct, but it leaves the partial-state risk unflagged
in the plan that an implementer will read. Subscribers that fan out into
multiple downstream dispatches deserve an explicit note about what happens
on mid-loop failure.

## Suggestion

Acknowledge the partial-failure case in the Error handling section so the
implementer understands the operational profile they're shipping:

> **Partial fan-out failure.** The subscriber dispatches 12 commands
> sequentially. If a dispatch fails mid-loop, the subscriber returns
> `EventError::SubscriberExecution` and the kernel retries the event. The
> already-persisted commands from the failed attempt remain in
> `command_entries` with their original random UUIDs; the retry produces a
> fresh set of 12 commands with new UUIDs. Until fan-out idempotency
> (deterministic command_ids — see S008/open question) lands, the
> `SourceRetailer` handlers must be idempotent themselves and the operator
> should expect duplicate `SourceRetailer` rows on transient infra errors.

Optionally, also add a verification step (deferrable along with S008) that
simulates a mid-loop failure and asserts the duplication shape, so the
behaviour is captured before idempotency lands.
