# S010 — `RetailerSourcingEvent` exposure through `io` is not specified

| Field                    | Value                                                                                                                                                                                                                                                                                                                   |
|--------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                                                                                                                    |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                                                                           |
| Decision                 | accepted                                                                                                                                                                                                                                                                                                                |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (`app_event.rs` block now defines `pub mod io { pub use super::RetailerSourcingEvent; }`; handler imports via `crate::app_event::io::RetailerSourcingEvent`; note added about integration tests using `crate::io::RetailerSourcingEvent`) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                                                              |
| Author                   | Claude Code, claude-sonnet-4-6, high                                                                                                                                                                                                                                                                                    |
| Reviewer                 |                                                                                                                                                                                                                                                                                                                         |

## Issue

The plan introduces `app_event.rs` with `pub enum RetailerSourcingEvent`,
and the file-by-file section says:

> `apps/retailer-sourcing/src/lib.rs` — `mod active_retailers; mod app_event;
> mod source_retailer;`, re-export each feature's `io`; **expose
> `RetailerSourcingEvent` via top-level `io`**.

But the `app_event.rs` code block does not define a `pub mod io` submodule,
and no `pub use` statement is shown anywhere that would actually export
`RetailerSourcingEvent`.

Worse, the handler example reaches *into the private module* directly:

```rust
use crate::app_event::RetailerSourcingEvent;
```

That bypasses the coding guideline rule that "external consumers — including
integration tests — import exclusively from `crate::io::*`. They never reach
into a feature module directly". The integration test in `tests/` will need
`RetailerSourcingEvent` (or at least the variant payload types) to assert
emitted events, and the only sanctioned route is through `crate::io`.

Two specific places need clarification:

1. Does `app_event.rs` follow the feature-module pattern (`pub mod io { pub
   use super::RetailerSourcingEvent; }`)?
2. Do downstream consumers (handler in `daily_sourcing.rs`, integration
   test) import via `crate::io::RetailerSourcingEvent` or via the private
   module path?

## Suggestion

Spell out the full surface explicitly. Update `app_event.rs` to match the
feature-module pattern:

```rust
// src/app_event.rs
use serde::{Deserialize, Serialize};

pub mod io {
    pub use super::RetailerSourcingEvent;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum RetailerSourcingEvent {
    DailySourcingRequested(crate::daily_sourcing::io::DailySourcingRequested),
}

impl kernel::ApplicationEvent for RetailerSourcingEvent { … }
```

Update the handler example to import via `crate::io` (since `lib.rs`
re-exports it):

```rust
use crate::app_event::io::RetailerSourcingEvent;
```

…or, if the handler is allowed to use sibling-module-internal paths because
both live inside the same crate, document that explicitly (and reference it
in coding-guidelines.md as the canonical exception).

Also update the verification section to mention that the integration test
asserts the deserialized event payload via the `crate::io::*` surface, so
no test reaches into the private `app_event` module.
