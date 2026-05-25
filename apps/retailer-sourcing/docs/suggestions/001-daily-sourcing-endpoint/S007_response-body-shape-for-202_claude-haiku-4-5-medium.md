# S004 - Response body shape for 202

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | medium                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Created at               | 2026-05-25                                                                        |
| Author                   | Claude Code, claude-haiku-4-5, medium                                             |
| Reviewer                 |                                                                                   |

## Issue

The verification section states that on success the response should contain
"a UUID `command_id`," but the plan doesn't specify the JSON shape. Bruno
collection's `daily-sourcing.bru` expects `body.command_id` to be a UUID
string, but the handler implementation detail (response struct, serialization
format) is left to the implementer. This risks inconsistency with the test
assertions.

## Suggestion

Add a response struct definition and serialization format to the
"Command envelope" section:

```rust
#[derive(Serialize)]
struct DailySourcingResponse {
    command_id: Uuid,
}

// Handler returns:
Ok((StatusCode::ACCEPTED, Json(DailySourcingResponse { command_id })))
```

This makes the response contract explicit and ensures the Bruno test matches
the implementation.
