# S002 — `correlation_id` in fan-out should propagate saga thread, not repeat `event_id`

| Field                    | Value                                                                                                                                                                                                                                                                |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                               |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                        |
| Decision                 | accepted                                                                                                                                                                                                                                                             |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Fan-out metadata now uses `correlation_id: meta.correlation_id` and `causation_id: Some(meta.event_id)`; verification test 3 updated to assert end-to-end correlation_id propagation) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                           |
| Author                   | Claude Code, claude-sonnet-4-6, medium                                                                                                                                                                                                                               |
| Reviewer                 |                                                                                                                                                                                                                                                                      |

## Issue

The fan-out subscriber sets both `correlation_id` and `causation_id` to
`envelope.metadata.event_id`:

```rust
correlation_id: Some(envelope.metadata.event_id),
causation_id: Some(envelope.metadata.event_id),
```

These two fields have distinct semantics:

- `causation_id` — the immediate cause of this command: the
  `DailySourcingRequested` event. Should be `meta.event_id`. ✓ (correct
  intent)
- `correlation_id` — the saga thread that started with the original HTTP
  trigger. Should be propagated from the event's own `correlation_id`, which
  was set by the HTTP handler to the `StartDailySourcing` command's
  `command_id`.

Setting `correlation_id = event_id` breaks tracing: every `SourceRetailer`
command will show a different saga root, making it impossible to correlate all
12 fan-out commands back to the originating HTTP request.

## Suggestion

Use `meta.correlation_id` for `correlation_id` and `meta.event_id` for
`causation_id`:

```rust
metadata: Some(NewCommandMetadata {
    command_id: Uuid::now_v7(),
    correlation_id: meta.correlation_id,
    causation_id: Some(meta.event_id),
    source: Some("event:DailySourcingRequested".to_string()),
}),
```

This threads the correlation_id set on the `StartDailySourcing` command
through `DailySourcingRequested` and into every `SourceRetailer` command,
forming a complete correlation chain for the full daily sourcing saga.

Also update the integration test in the verification section (test 3) to
assert that the `correlation_id` on each `SourceRetailer` row matches the
`correlation_id` on the original `StartDailySourcing` command — not the
`DailySourcingRequested` event_id.
