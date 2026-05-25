# S009 - Route composition in app() is broken

| Field                    | Value                                                                                                                                                                                                                   |
|--------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                    |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                       |
| Decision                 | accepted                                                                                                                                                                                                                |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (Feature module rewritten to use `register` pattern; `app()` updated to compose features without conflicting `.nest("/", ...)` calls) |
| Created at               | 2026-05-25                                                                                                                                                                                                              |
| Author                   | Claude Code, claude-opus-4-7, high                                                                                                                                                                                      |
| Reviewer                 |                                                                                                                                                                                                                         |

## Issue

The proposed `lib.rs::app()` body composes routes by nesting two sub-routers
under the same mount point:

```rust
pub fn app(gateway: Arc<CommandGateway>) -> Route {
    Route::new()
        .nest("/", check_health::route())
        .nest("/", daily_sourcing::route(gateway))
}
```

In Poem, `Route::nest(path, ep)` mounts a sub-router under a path prefix. Two
`.nest("/", ...)` calls on the same `Route` register two endpoints at the
same wildcard mount and the second silently replaces the first — or panics on
overlap, depending on the version. Either way, the health endpoint is lost
once `daily-sourcing` is added.

This is a latent bug that won't be caught until the integration test for
`/health` and the integration test for `/daily-sourcing/` both run against
the same `app()`. The current health integration test only runs the
health-only `check_health::route()` — it would pass even with this broken
top-level composition.

## Suggestion

Each feature module should expose its endpoint(s) and the composer in
`lib.rs::app()` should `.at()` each path on a single shared `Route`, or
features should accept a builder/router and register onto it. Concrete shape:

```rust
// each feature exposes its handler(s) and path constant, not a full Route
pub fn app(gateway: Arc<CommandGateway>, bearer_token: Arc<String>) -> Route {
    Route::new()
        .at(check_health::io::HEALTH_CHECK_PATH, get(check_health::handler))
        .at(
            daily_sourcing::io::DAILY_SOURCING_PATH,
            post(daily_sourcing::handler)
                .with(AddData::new(gateway))
                .with(BearerAuth::new(bearer_token)),
        )
}
```

Or, if features should encapsulate their own middleware:

```rust
// daily_sourcing.rs exposes a fn that takes a Route and registers itself
pub fn register(route: Route, gateway: Arc<CommandGateway>, bearer: Arc<String>) -> Route {
    route.at(io::DAILY_SOURCING_PATH, post(handler).with(AddData::new(gateway)).with(BearerAuth::new(bearer)))
}

// lib.rs
pub fn app(gateway: Arc<CommandGateway>, bearer: Arc<String>) -> Route {
    let mut route = Route::new();
    route = check_health::register(route);
    route = daily_sourcing::register(route, gateway, bearer);
    route
}
```

Either pattern works; pick one and update the plan's example. Also add an
integration test that mounts the full `app()` and verifies both `/health` and
`/daily-sourcing/` respond — without that, this bug class recurs.

## Decision

Features should encapsulate their own middleware