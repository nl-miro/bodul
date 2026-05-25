# S005 - Startup dependencies clarification

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | low                                                                               |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Created at               | 2026-05-25                                                                        |
| Author                   | Claude Code, claude-haiku-4-5, low                                                |
| Reviewer                 |                                                                                   |

## Issue

The "Wiring at startup" section refers to "the mulac kernel persistent
startup path" but doesn't link to where that's documented or provide an
example. An implementer unfamiliar with mulac may search for it and find
nothing, or assume they need to assemble the components manually. This adds
friction during implementation.

## Suggestion

Add a reference link or at least a file path hint. For example:

> See `~/.cargo/git/checkouts/mulac-*/libs/mulac_diesel/examples/` for a
> reference implementation of the kernel startup pattern.

If no good example exists in mulac yet, note that as an open question.
This unblocks the implementer without requiring a full code sketch.
