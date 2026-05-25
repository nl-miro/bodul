# S003 - Test fixture for integration tests

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

The verification section mentions "the test sets up an in-memory or test-schema
Postgres (via a fixture helper to be added once mulac's recommended test
harness is identified)." This defers the test infrastructure decision,
leaving the implementer blocked on determining how to wire the database for
tests. Without clear guidance, tests may be skipped or written in a way that
doesn't catch real failures (e.g., mocking the store instead of hitting
Postgres).

## Suggestion

Add a temporary note in the "Verification" section clarifying the immediate
path forward:

> **Test database:** For now, use testcontainers-rs to spin up a Postgres
> container per test run. Once mulac's recommended test harness is
> identified, migrate tests to that pattern. This ensures tests run against
> real schema and catch actual query failures.

This unblocks implementation and documents the temporary decision, making it
easy to refactor later.
