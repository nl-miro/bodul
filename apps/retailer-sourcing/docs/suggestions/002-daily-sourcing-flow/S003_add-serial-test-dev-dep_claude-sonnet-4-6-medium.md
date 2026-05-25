# S003 — Add `serial_test` dev dependency for `#[serial]` tests

| Field                    | Value                                                                                                                                                                                                                        |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                       |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                |
| Decision                 | accepted                                                                                                                                                                                                                     |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (`Cargo.toml` file-by-file entry now includes `serial_test = "3"` in dev-dependencies; integration-test entry notes `use serial_test::serial`) |
| Created at               | 2026-05-25                                                                                                                                                                                                                   |
| Author                   | Claude Code, claude-sonnet-4-6, medium                                                                                                                                                                                       |
| Reviewer                 |                                                                                                                                                                                                                              |

## Issue

The verification section marks integration tests with `#[serial]` to
serialise access to the shared database, but the plan never mentions adding the
`serial_test` crate to `[dev-dependencies]`. Without it the attribute is
unresolved and the crate won't compile.

## Suggestion

Add to the file-by-file section:

> `apps/retailer-sourcing/Cargo.toml` — add `serial_test = "3"` to
> `[dev-dependencies]`.

And add the following import to the test file section:

```rust
use serial_test::serial;
```
