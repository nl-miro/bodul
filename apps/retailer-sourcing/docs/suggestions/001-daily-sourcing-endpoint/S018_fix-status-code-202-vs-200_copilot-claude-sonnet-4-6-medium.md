# S018 - Fix status code 202 — show correct Poem API

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | medium                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | rejected                                                                          |
| Implementation reference |                                                                                   |
| Created at               | 2026-05-25                                                                        |
| Author                   | GitHub Copilot, claude-sonnet-4.6, medium                                         |
| Reviewer                 | GitHub Copilot, claude-sonnet-4.6                                                 |

## Issue

The plan consistently says the endpoint returns "202 Accepted". The code sketch uses
`(StatusCode::ACCEPTED, Json(DailySourcingResponse { command_id }))` but the rest
of the plan (test assertions, Bruno collection) references 202 without showing
the exact Poem return type.

In practice `poem::web::Json<T>` alone returns **200 OK** by default. The tuple
`(StatusCode, Json<T>)` form is required to get 202, and the handler's declared
return type must change accordingly:

```rust
// 200 OK — NOT what the plan describes:
Ok(poem::web::Json(DailySourcingResponse { command_id }))

// 202 Accepted — what the plan describes:
Ok((StatusCode::ACCEPTED, poem::web::Json(DailySourcingResponse { command_id })))
```

The current implementation (commit 013880c) returns 200 because `poem::web::Json`
defaults to 200, and the integration tests assert 200. Plan and implementation
disagree on the status code.

## Suggestion

Make the plan the single source of truth. Decide on one status code and reflect it
exactly in the handler type signature, code sketch, test assertions, and Bruno
collection.

**Recommended: use 200 OK.** The 202 vs 200 distinction matters when the caller
needs to distinguish "accepted but not yet processed" from "fully handled." Here the
response already includes `command_id` as proof of enqueue — 200 with the command_id
body is self-documenting. Using 202 requires a tuple return type, a different test
assertion (`assert_status(StatusCode::ACCEPTED)`), and a Bruno assertion change.

Update the plan to say 200 throughout and show the simple return:

```rust
// handler return type:
poem::Result<poem::web::Json<io::DailySourcingResponse>>

// at the end of the handler:
Ok(poem::web::Json(io::DailySourcingResponse { command_id }))
```

Update test assertion to `assert_status_is_ok()` / `StatusCode::OK`.
