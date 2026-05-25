# Coding Guidelines

These conventions apply across the Bodul repository. They describe how crates
are laid out, how modules expose their public surface, and how features are
organized inside service crates.

## Repository layout

- `apps/` is a Cargo workspace containing service binaries.
- `lib/` is a separate Cargo workspace containing shared libraries.
- Each workspace has its own `Cargo.lock`. Do not list `lib/*` crates as
  members of the `apps/` workspace; if an app needs a lib crate, depend on it
  by `path = "../../lib/<name>"`.

## Per-crate scaffolding

Every crate (under `apps/` or `lib/`) must contain:

- `Makefile` with `fmt`, `check`, and `test` targets that shell out to
  `cargo fmt`, `cargo check`, `cargo test`.
- `docs/` directory for crate-specific documentation. Keep a `.gitkeep` in it
  while empty.

The root `Makefile` aggregates `fmt`, `check`, and `test` across every crate
in both workspaces — keep it in sync when adding a new crate.

## Module organization in service crates

Service crates (e.g. `apps/retailer-sourcing`) are organized by **feature**,
not by technical layer. One file per feature.

### Layout

```
src/
  lib.rs              // composes features, exposes `app()` and `pub mod io`
  bin/<service>.rs    // thin binary that calls `lib::app()`
  <feature>.rs        // everything for a feature lives here
tests/
  <feature>.rs        // integration test per feature, named to match
```

### Feature module rules

1. **The feature module is private** in `lib.rs` (`mod <feature>;`, not
   `pub mod`). Internals stay internal.
2. **Each feature module declares its own `pub mod io`** containing only the
   items that callers outside the crate need (route paths, request/response
   DTOs, error types, etc.).
3. **The crate's top-level `pub mod io`** in `lib.rs` re-exports every
   feature's `io` module:
   ```rust
   pub mod io {
       pub use crate::<feature>::io::*;
   }
   ```
   External consumers — including integration tests — import exclusively from
   `crate::io::*`. They never reach into a feature module directly.
4. **`lib.rs` exposes a `pub fn app() -> Route`** that composes the routes
   from each feature's `route()` constructor. The binary is a thin wrapper
   around `app()`.

### Example

```rust
// src/lib.rs
mod check_health;

use poem::Route;

pub mod io {
    pub use crate::check_health::io::*;
}

pub fn app() -> Route {
    check_health::route()
}
```

```rust
// src/check_health.rs
use poem::{Route, get, handler, web::Json};

pub mod io {
    pub const HEALTH_CHECK_PATH: &str = "/health";
}

#[handler]
fn health_check() -> Json<...> { /* ... */ }

pub fn route() -> Route {
    Route::new().at(io::HEALTH_CHECK_PATH, get(health_check))
}
```

## Integration tests

- Live in `tests/<feature>.rs` (e.g. `tests/check_health.rs`).
- Drive the real router via `retailer_sourcing::app()` and the public
  contract from `retailer_sourcing::io`.
- Use Poem's `TestClient` (enable the `test` feature on `poem` in
  `[dev-dependencies]`) rather than spawning the binary.
- Never import from a feature module directly; if a test needs something,
  expose it through that feature's `io` submodule.

## Rust code style

### Imports in code and tests

Always import individual bindings explicitly rather than using them fully
qualified inline. Prefer:

```rust
use poem::{Route, handler, web::Json};

fn example() -> Json<Foo> { … }
```

over:

```rust
fn example() -> poem::web::Json<Foo> { … }
```

This keeps call sites readable, makes the dependency surface visible at a
glance, and reduces noise in function signatures and expressions.

Additional formatting rules for import blocks:

- No empty lines between import statements.
- Multi-line imports must have a trailing `//` comment after the last binding
  and the closing `}` on its own line. This prevents `rustfmt` from collapsing
  the group to a single line, keeping one binding per line:

  ```rust
  use poem::{
      EndpointExt,
      Route,
      handler, //
  };
  ```

- Each line in a multi-line import must be at most 80 characters.



Format all markdown text (`.md` files) with lines no longer than 80 characters
wide. Wrap at word boundaries. This makes diffs readable, enables easy
terminal editing, and keeps line-based review tools (git blame, grep)
predictable. Code blocks and tables are exempt from this rule if wrapping
would harm readability.
