# S003 - Specify database test harness

| Field                    | Value                                                                                                                                                                           |
|--------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                          |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                               |
| Decision                 | accepted                                                                                                                                                                        |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (Verification section expanded with test database strategy using Docker Compose and Postgres) |
| Created at               | 2026-05-25                                                                                                                                                                      |
| Author                   | Codex, gpt-5, medium                                                                                                                                                            |
| Reviewer                 | Copilot (staff review)                                                                                                                                                          |

## Issue
The verification section still defers the database test setup to a future investigation. It mentions either in-memory or test-schema Postgres and a fixture helper, but does not define a concrete isolation and reset strategy for CI.

## Suggestion
Choose one repeatable Postgres test strategy and document it in the plan before implementation starts. For example, use a dedicated test database schema or temporary database per test run, plus a helper that truncates the command tables before each case. Record the fixture approach in the verification section so the acceptance test can be implemented without further design work.

## Decision

We should be using Postgres for test, dev and prod environments. Using docker, set up a test database and rabbitmq like it's done in mulac (https://github.com/nulllabsdev/mulac/blob/main/test_apps/docker-compose.yml)
