# S001 - Clarify bootstrap ownership

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | high                                                                              |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | deffered                                                                          |
| Implementation reference |                                                                                   |
| Created at               | 2026-05-25                                                                        |
| Author                   | Codex, gpt-5, high                                                                |
| Reviewer                 |                                                                                   |

## Issue
The plan describes three different owners for the same startup responsibility: the binary building the pool and gateway directly, the binary calling `app(...)` with gateway/token parameters, and the mulac kernel persistent startup path. That leaves the implementation boundary ambiguous.

## Suggestion
Pick one startup ownership model and document it consistently. Prefer the mulac kernel persistent startup path as the single place that builds the pool, recorder, and two-phased gateway, then pass the kernel-provided gateway and token into the service app. Update the design, wiring, and file-by-file sections so they all describe the same flow.
