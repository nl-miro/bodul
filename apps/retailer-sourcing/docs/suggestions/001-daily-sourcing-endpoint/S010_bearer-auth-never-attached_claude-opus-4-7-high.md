# S010 - Bearer auth middleware is defined but never attached

| Field                    | Value                                                                                                                                                                                                                              |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                               |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                                  |
| Decision                 | accepted                                                                                                                                                                                                                           |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (`daily_sourcing::register` now accepts bearer token and attaches `BearerAuth`; `app()` updated to accept and thread `Arc<String>` bearer token) |
| Created at               | 2026-05-25                                                                                                                                                                                                                         |
| Author                   | Claude Code, claude-opus-4-7, high                                                                                                                                                                                                 |
| Reviewer                 |                                                                                                                                                                                                                                    |

## Issue

The plan defines a `BearerAuth` middleware in detail, reads the token in the
startup section (step 4: "Reading `HARDCODED_BEARER_TOKEN`"), and lists the
constant `HARDCODED_BEARER_TOKEN` in the configuration table — but the token
never flows into the application. Specifically:

1. `daily_sourcing::route(gateway: Arc<CommandGateway>) -> Route` only takes
   a gateway. It does not take the bearer token.
2. `lib.rs::app(gateway: Arc<CommandGateway>) -> Route` only takes a gateway.
3. No `.with(BearerAuth::new(...))` appears in any code snippet.
4. The "attach via `.with(BearerAuth::new(token))`" snippet that previously
   appeared in the bearer-auth section was removed during refactoring.

If this were implemented as written, the `/daily-sourcing/` endpoint would
accept any request — the bearer check would be dead code. The
`daily-sourcing-missing-auth` and `daily-sourcing-wrong-token` Bruno tests
would fail in production while the happy-path test passes.

## Suggestion

Restore an explicit attachment step in the Bearer authentication section
showing where the middleware enters the route tree. Extend the relevant
signatures:

- `daily_sourcing::route` (or `register`) accepts a `bearer_token: Arc<String>`.
- `lib.rs::app` accepts the same.
- The startup wiring section explicitly says "pass the token into `app()`".

Add a snippet like:

```rust
Route::new().at(io::DAILY_SOURCING_PATH, post(trigger))
    .with(AddData::new(gateway))
    .with(BearerAuth::new(bearer_token))
```

Also update the "File-by-file changes" entry for `lib.rs` from "change
`app()` signature to accept the gateway + token" — that wording is correct,
but the example code in the plan still shows a single-arg `app(gateway)`.
Make the example match the description.

Tie this to S009: the same fix can land in one edit if both are accepted.
