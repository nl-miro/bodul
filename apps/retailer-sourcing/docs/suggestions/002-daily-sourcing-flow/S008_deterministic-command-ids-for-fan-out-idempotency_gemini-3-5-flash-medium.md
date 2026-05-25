# S008 — Deterministic command IDs for fan-out idempotency

| Field                    | Value                                                                         |
|--------------------------|-------------------------------------------------------------------------------|
| Priority                 | medium                                                                        |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` |
| Decision                 | deffered                                                                      |
| Implementation reference |                                                                               |
| Created at               | 2026-05-25                                                                    |
| Author                   | Gemini, gemini-3.5-flash, medium                                              |
| Reviewer                 |                                                                               |

## Issue

In `DailySourcingFanOutSubscriber::handle`, the subscriber generates a random `Uuid::now_v7()` for each dispatched `SourceRetailer` command:

```rust
let gateway_envelope = NewCommandEnvelope {
    command: NewCommand { ... },
    metadata: Some(NewCommandMetadata {
        command_id: Uuid::now_v7(),
        ...
    }),
};
```

If the subscriber is retried (e.g., due to a transient database failure midway through dispatching, an unexpected crash, or a duplicate delivery of the `DailySourcingRequested` event), it will generate completely new UUIDs. This will cause duplicate `SourceRetailer` commands to be successfully inserted into `command_entries` and subsequently processed multiple times.

The "Open questions" section identifies "Fan-out idempotency" as deferred/open, but a concrete solution is straightforward to integrate directly into the fan-out loop.

## Suggestion

Generate deterministic UUIDs for each fan-out `SourceRetailer` command by using UUID v5 namespaced on the unique `DailySourcingRequested` `event_id`, the `retailer_code`, and the `method`.

Because the DB enforces a uniqueness constraint on the primary key of `command_entries`, retrying the subscriber will generate identical `command_id`s, which will safely fail to insert (or be ignored) on duplicates, rendering the entire fan-out process fully idempotent and robust.

Update the `DailySourcingFanOutSubscriber` loop:

```rust
// Use a fixed namespace UUID for Daily Sourcing Fan-out
const FAN_OUT_NAMESPACE: Uuid = Uuid::from_u128(0x6ba7b810_9dad_11d1_80b4_00c04fd430c8); // or any unique static UUID

for retailer_code in &event.retailer_codes {
    for method in SourcingMethod::all() {
        let cmd = SourceRetailer { ... };
        
        // Generate a deterministic command_id
        let id_input = format!("{}:{}:{:?}", meta.event_id, retailer_code.as_string(), method);
        let command_id = Uuid::new_v5(&FAN_OUT_NAMESPACE, id_input.as_bytes());

        let gateway_envelope = NewCommandEnvelope {
            command: NewCommand { ... },
            metadata: Some(NewCommandMetadata {
                command_id,
                correlation_id: meta.correlation_id,
                causation_id: Some(meta.event_id),
                source: Some("event:DailySourcingRequested".to_string()),
            }),
        };
        
        // Dispatch (and ignore or handle duplicate key errors if necessary)
        self.command_gateway.dispatch(gateway_envelope)...
    }
}
```
