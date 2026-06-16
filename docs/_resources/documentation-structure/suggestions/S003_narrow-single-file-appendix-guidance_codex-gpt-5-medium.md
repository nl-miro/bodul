# S003 - Narrow single-file appendix guidance

| Field                    | Value                             |
|--------------------------|-----------------------------------|
| Priority                 | low                               |
| File                     | `docs/documentation-structure.md` |
| Decision                 | refused                           |
| Implementation reference |                                   |
| Created at               | 2026-06-15                        |
| Author                   | Codex, gpt-5, medium              |
| Reviewer                 |                                   |

## Issue

The `Lighter Alternative (Single-File Documents)` section recommends putting the
research trail inside the deliverable. That is useful in constrained contexts,
but it weakens the suggestion-driven principle that review scaffolding should
stay outside the reader-facing document by default.

## Suggestion

Rewrite the single-file alternative as an exception, not a peer default. State
that it should be used only when a separate `_resources/` folder is impossible
or too heavy for the document's lifecycle. Also mention that suggestion
documents remain preferred when an LLM or reviewer is producing actionable
feedback.
