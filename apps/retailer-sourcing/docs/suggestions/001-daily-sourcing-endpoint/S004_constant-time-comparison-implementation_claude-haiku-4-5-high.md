# S001 - Constant-time comparison implementation

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | high                                                                              |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | deffered                                                                          |
| Implementation reference |                                                                                   |
| Created at               | 2026-05-25                                                                        |
| Author                   | Claude Code, claude-haiku-4-5, high                                               |
| Reviewer                 |                                                                                   |

## Issue

The bearer auth middleware sketch references `constant_time_eq()` but doesn't
specify the implementation. The plan defers to `subtle::ConstantTimeEq` or
"a small local helper" without guidance. For security-sensitive code, this
ambiguity risks developers choosing the wrong approach or forgetting the
check entirely during implementation.

## Suggestion

Add a concrete code snippet in the bearer auth section showing the
constant-time comparison implementation. Recommend using `subtle` crate for
production code (well-audited, standard practice) rather than writing a
local helper, which is prone to timing-side-channel bugs.

Example addition after the middleware sketch:

```rust
// Use the subtle crate for constant-time comparison.
// Add to Cargo.toml: subtle = "2"

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    use subtle::ConstantTimeEq;
    a.ct_eq(b).into()
}
```

This removes ambiguity and prevents security regressions.
