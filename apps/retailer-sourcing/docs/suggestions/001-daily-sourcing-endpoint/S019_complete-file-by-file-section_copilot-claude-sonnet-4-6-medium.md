# S019 - Complete the file-by-file section

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | medium                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | commit 013880c                                                                    |
| Created at               | 2026-05-25                                                                        |
| Author                   | GitHub Copilot, claude-sonnet-4.6, medium                                         |
| Reviewer                 | GitHub Copilot, claude-sonnet-4.6                                                 |

## Issue

The "File-by-file changes" section lists 5 files but is missing 4 that are
required and were only discovered during implementation:

1. **`apps/retailer-sourcing/migrations/…/up.sql`** — the mulac infrastructure
   tables (`command_entries`, `event_entries`, `inbox_entries`, `outbox_entries`)
   must be migrated into the service's own database. Without this file, the
   kernel panics on startup. The plan mentions "Diesel wiring (migration runner)"
   vaguely but never lists the migration directory as a deliverable.

2. **`apps/retailer-sourcing/tests/utils/mod.rs`** — Rust integration tests cannot
   share helpers via a top-level `tests/utils.rs` (it gets compiled as its own
   test binary). The shared pool, `run_migrations`, `reset_tables`, and kernel
   setup must live in `tests/utils/mod.rs`. This pattern was discovered by
   reading the mulac twitter test app during implementation.

3. **`apps/retailer-sourcing/tests/check_health.rs`** — `app()` changes its
   signature from `app()` to `app(state, bearer)`. The existing health test calls
   `app()` with no arguments and will fail to compile. This is a breaking change
   to an existing test and must be called out explicitly.

4. **`apps/retailer-sourcing/docker-compose.test.yml`** — mentioned in the
   Verification section but absent from the file-by-file list, creating a gap
   between what the plan promises and what it instructs implementors to create.

## Suggestion

Add the four missing entries to the file-by-file section:

- `apps/retailer-sourcing/migrations/YYYY-MM-DD-000001_infrastructure/up.sql` —
  copy the mulac infra tables SQL from
  `test_apps/twitter/migrations/…/infrastructure/up.sql`. Also add `down.sql`.
- `apps/retailer-sourcing/tests/utils/mod.rs` — shared test helpers: pool
  (`OnceLock`), `run_migrations`, `reset_tables`, `setup() → (DbPool, Arc<PersistentKernelState>)`.
- `apps/retailer-sourcing/tests/check_health.rs` — update to call
  `app(state, Arc::new("unused".to_string()))` using `utils::setup()`.
- `apps/retailer-sourcing/docker-compose.test.yml` — already described in
  Verification; just list it here so the checklist is complete.
