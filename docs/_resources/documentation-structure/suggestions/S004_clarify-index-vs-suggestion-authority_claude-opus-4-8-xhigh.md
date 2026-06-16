# S004 - Clarify INDEX vs suggestion authority

| Field                    | Value                                                                                      |
|--------------------------|--------------------------------------------------------------------------------------------|
| Priority                 | low                                                                                        |
| File                     | `docs/documentation-structure.md`                                                          |
| Decision                 | completed                                                                                  |
| Implementation reference | `docs/_resources/documentation-structure/proposals/documentation-structure-proposal-v1.md` |
| Created at               | 2026-06-15                                                                                 |
| Author                   | Claude Code, claude-opus-4-8, xhigh                                                        |
| Reviewer                 | User                                                                                       |

## Issue

After adding the "Suggestion and Proposal Workflow" section, the guide now
records suggestion decisions in two places: each suggestion document's
`Decision` field, and the `INDEX.md` "Decision Log". The Traceability bullet
says `INDEX.md` "records rejected or deferred suggestions in the decision log."
This risks divergence — a rejected suggestion's status would live both in its
own document and, restated, in `INDEX.md`, with no defined source of truth.

## Suggestion

Clarify that the suggestion documents are the authoritative record of each
suggestion's decision, and that `INDEX.md` only *links to / summarizes* them
rather than re-recording the decision. Reword the Traceability bullet from
"records rejected or deferred suggestions" to "links to rejected or deferred
suggestions," so there is a single source of truth.
