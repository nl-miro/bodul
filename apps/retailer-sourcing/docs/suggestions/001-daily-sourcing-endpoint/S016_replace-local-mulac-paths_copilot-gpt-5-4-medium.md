# S016 - Replace machine-local mulac references

| Field                    | Value                                                                                                                                                                                                       |
|--------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                      |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                           |
| Decision                 | accepted                                                                                                                                                                                                    |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (All `~/.cargo/git/checkouts/mulac-*/...` paths replaced with `https://github.com/nulllabsdev/mulac/blob/main/...` links) |
| Created at               | 2026-05-25                                                                                                                                                                                                  |
| Author                   | GitHub Copilot, gpt-5.4, medium                                                                                                                                                                             |
| Reviewer                 |                                                                                                                                                                                                             |

## Issue

The startup and verification sections point implementers at
`~/.cargo/git/checkouts/mulac-*/...` paths for the kernel startup example and
test service layout. Those references depend on a developer having a specific
local cargo checkout shape and do not work in code review, CI, or on a fresh
machine.

That makes the plan harder to follow and undermines one of the main goals of an
implementation plan: giving any engineer a reproducible path to execute the
work.

## Suggestion

Replace machine-local cargo-checkout paths with stable references.

Prefer one of these:

1. GitHub links to the exact mulac files being referenced.
2. Repo-local notes that summarize the required setup and point to pinned mulac
   paths or tags.
3. A checked-in proposal or follow-up setup document in this repository if the
   mulac examples are expected to drift.

If the plan depends on a specific mulac revision, record the commit SHA or tag
alongside the reference so the example remains reproducible.

## Decision

Use Github links