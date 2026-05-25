# S001 — `NewEventEnvelope.metadata` is `Option` — subscriber code won't compile

| Field                    | Value                                                                                                                                                                                                              |
|--------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                               |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                      |
| Decision                 | accepted                                                                                                                                                                                                           |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Fan-out subscriber updated to unwrap `envelope.metadata` via `as_ref().ok_or_else(...)` before reading `event_id`/`correlation_id`) |
| Created at               | 2026-05-25                                                                                                                                                                                                         |
| Author                   | Claude Code, claude-sonnet-4-6, high                                                                                                                                                                               |
| Reviewer                 |                                                                                                                                                                                                                    |

## Issue

The `DailySourcingFanOutSubscriber` code in the plan accesses
`envelope.metadata.event_id` as if `metadata` is a plain struct field:

```rust
correlation_id: Some(envelope.metadata.event_id),
causation_id: Some(envelope.metadata.event_id),
```

But the actual `NewEventEnvelope` type (verified in mulac source) is:

```rust
pub struct NewEventEnvelope {
    pub event_type: String,
    pub payload: String,
    pub metadata: Option<NewEventMetadata>,
}
```

`metadata` is `Option<NewEventMetadata>`. Accessing `.event_id` on it directly
is a compile error.

## Suggestion

Replace the two metadata field accesses with explicit `Option` unwrapping.
Since a `DailySourcingRequested` event written by a proper handler will always
carry metadata, unwrapping with `ok_or` and mapping to
`EventError::SubscriberExecution` is the safest pattern:

```rust
let meta = envelope
    .metadata
    .as_ref()
    .ok_or_else(|| {
        EventError::SubscriberExecution(
            "DailySourcingRequested event is missing metadata".to_string(),
        )
    })?;
```

Then use `meta.event_id` for `causation_id` and `meta.correlation_id` for
`correlation_id` (see also S002).

Update the fan-out loop accordingly:

```rust
impl EventSubscriberPort for DailySourcingFanOutSubscriber {
    fn handle(&self, envelope: &NewEventEnvelope) -> Result<(), EventError> {
        let meta = envelope
            .metadata
            .as_ref()
            .ok_or_else(|| {
                EventError::SubscriberExecution(
                    "DailySourcingRequested event is missing metadata"
                        .to_string(),
                )
            })?;
        let event: DailySourcingRequested =
            serde_json::from_str(&envelope.payload)
                .map_err(|e| {
                    EventError::SubscriberExecution(e.to_string())
                })?;

        for retailer_code in &event.retailer_codes {
            for method in SourcingMethod::all() {
                let cmd = SourceRetailer { … };
                let gateway_envelope = NewCommandEnvelope {
                    command: NewCommand { … },
                    metadata: Some(NewCommandMetadata {
                        command_id: Uuid::now_v7(),
                        correlation_id: meta.correlation_id,
                        causation_id: Some(meta.event_id),
                        source: Some(
                            "event:DailySourcingRequested".to_string(),
                        ),
                    }),
                };
                …
            }
        }
        Ok(())
    }
}
```
