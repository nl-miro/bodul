# S002 - Add suggestion and proposal lifecycle

| Field                    | Value                                                                                      |
|--------------------------|--------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                     |
| File                     | `docs/documentation-structure.md`                                                          |
| Decision                 | completed                                                                                  |
| Implementation reference | `docs/_resources/documentation-structure/proposals/documentation-structure-proposal-v1.md` |
| Created at               | 2026-06-15                                                                                 |
| Author                   | Codex, gpt-5, medium                                                                       |
| Reviewer                 | User                                                                                       |

## Issue

The structure guide describes resources, reviews, drafts, and an `INDEX.md`, but
it does not describe how suggestion documents and proposal copies fit into the
resource trail. This misses the core mechanism from
`docs/suggestion-driven-documentation.md`: proposed changes are captured first,
reviewed, applied to proposal copies, and then copied back to the deliverable.

## Suggestion

Add a short section named `Suggestion and Proposal Workflow` that states:

- Actionable review feedback goes into `_resources/{document-name}/suggestions/`.
- Proposal copies go into `_resources/{document-name}/proposals/`.
- The deliverable is updated only after pending suggestions are accepted,
  rejected, or deferred.
- `INDEX.md` should link accepted suggestions to implementation references and
  record rejected or deferred suggestions in the decision log.

This gives reviewers and LLM tools a clear path from feedback to controlled
document updates.
