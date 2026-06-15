# S004 - Clarify implementation reference timing

| Field                    | Value                                      |
|--------------------------|--------------------------------------------|
| Priority                 | medium                                     |
| File                     | `docs/suggestion-driven-documentation.md`  |
| Decision                 | pending                                    |
| Implementation reference |                                            |
| Created at               | 2026-06-15                                 |
| Author                   | Claude Code, claude-opus-4-8, xhigh        |
| Reviewer                 |                                            |

## Issue

`Implementation reference` is described (line 51) as a link to "the commit, pull
request, or document change that implemented the suggestion." But "Applying
suggestions" step 5 says to fill in the field and *then* commit ("fill in the
`Implementation reference` field, and commit before moving to the next
suggestion"). At fill time the implementing commit does not exist yet, so the
field cannot hold that commit's hash without a follow-up amend — a chicken-and-egg
ordering. (This already surfaced when applying the `documentation-structure`
suggestions, where the field was set to the proposal filename instead of a
commit.)

## Suggestion

Resolve the ordering explicitly, choosing one convention:

- (a) Define `Implementation reference` as the proposal artifact the change was
  applied to (e.g. `document-proposal-v1.md`), which is known at fill time; or
- (b) Keep it as the commit/PR and move the fill to a follow-up step (amend or a
  later commit) once the hash exists.

State the chosen convention so references are consistent across suggestions.
