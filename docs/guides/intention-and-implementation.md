# Intention and Implementation

This guide names the principle behind the feature-per-file layout that
[`docs/coding-guidelines.md`](../coding-guidelines.md) describes, and the
module-ordering convention that follows from it. It is the *why*; the
guidelines file is the *what*.

For a copy-and-rename starter, see
[`docs/templates/command_feature.rs.txt`](../templates/command_feature.rs.txt).

## Principle

A feature module has two halves:

- **Intention** — the surface the rest of the codebase is allowed to see and
  depend on. Lives in `pub mod io` at the top of the feature file.
- **Implementation** — HTTP routing, command/event handlers, subscribers,
  command/event types, pure domain types. Lives in *private* sibling
  modules under `io`.

External callers — other crates, other features inside the same crate, and
integration tests in `tests/` — only ever reach `crate::io::*`. They never
import from a feature's private mods directly. This is what lets the
implementation be rearranged, renamed, or replaced without ripple across the
codebase.

## Module ordering inside a feature file

Read a feature file top-to-bottom in this order:

1. `pub mod io { ... }` — the intention.
2. **Services exposed outside** — `mod http`, `mod handler`, `mod eventing`.
   Any combination the feature needs. These are the things `io` re-exports
   and the rest of the system calls into.
3. **Models the services operate on** — `mod application`. Command structs,
   event structs, error enums, `COMMAND_TYPE` and `EVENT_TYPE_*` constants.
4. **Pure types** — `mod domain`. No infrastructure dependencies. Often
   empty in early features; add types here as soon as the feature grows
   logic that does not belong in `application`.

Rationale: a reader scanning from the top sees the contract first, then how
it is served, then the data shapes it manipulates.

## Role of each mod

| Mod           | What lives here                                                                                         |
|---------------|---------------------------------------------------------------------------------------------------------|
| `io`          | Re-exports of the items external callers need. Nothing else.                                            |
| `http`        | Route path constants, request/response DTOs, `#[handler]` fns, `ResponseError` impls, `pub fn register` |
| `handler`     | `CommandHandlerPort` / `EventHandlerPort` implementations                                               |
| `eventing`    | `EventSubscriberPort` implementations (e.g. fan-out subscribers)                                        |
| `application` | Command structs, event structs, error enums, type-name constants                                        |
| `domain`      | Pure types with no infrastructure dependencies                                                          |

Every mod except `io` is private to the feature.

## Re-export discipline in `io`

List the exports explicitly:

```rust
pub mod io {
    pub use super::application::{
        COMMAND_TYPE,
        EVENT_TYPE_DAILY_SOURCING_REQUESTED,
        DailySourcingRequested,
        StartDailySourcing, //
    };
    pub use super::handler::DailySourcingHandler;
    pub use super::http::{DAILY_SOURCING_PATH, DailySourcingResponse, register};
}
```

Avoid `pub use super::http::*`. The whole point of `io` is that the public
surface is visible at a glance — a glob defeats that.

## Crate-level aggregation

The crate's top-level `pub mod io` in `lib.rs` re-exports each feature's
`io::*`, and the binary calls a single `app()` constructor that composes
feature routes. See
[`docs/coding-guidelines.md` § "Module organization in service crates"](../coding-guidelines.md#module-organization-in-service-crates)
for the exact shape — not repeated here.

## Tests and external crates

Integration tests in `tests/<feature>.rs` import from `crate::io::*` only.
If a test needs something that is not yet exported, expose it through the
feature's `io` submodule rather than reaching past it. The same rule
applies to other crates that depend on this one.

## Picking shapes

Not every feature needs every mod. Use the template as the default
starting point and delete what you do not need:

- A health-check style endpoint needs only `io` + `http`.
- An event subscriber that dispatches new commands needs `io` +
  `application` + `eventing`.
- A reporting/CLI utility needs `io` + `application` (+ `domain` if it
  grows logic).

When a feature outgrows a single file, promote it to a folder
(`<feature>/mod.rs` + `application.rs` + `http.rs` + `domain/`). Each
submodule still declares its own `pub mod io`, and the folder's `mod.rs`
re-exports them into a single feature `io`. See
`apps/retailer-sourcing/src/assembly/` for a worked example.

## See also

- [`docs/coding-guidelines.md`](../coding-guidelines.md) — repository
  layout, per-crate scaffolding, import style.
- [`docs/templates/command_feature.rs.txt`](../templates/command_feature.rs.txt)
  — copy-and-rename skeleton for an HTTP-triggered command/event feature.
- `apps/retailer-sourcing/src/daily_sourcing.rs` — the worked reference
  the template is modeled on.
