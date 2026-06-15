# S003 - Add decision/triage step

| Field                    | Value                                                                                                      |
|--------------------------|------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                     |
| File                     | `docs/suggestion-driven-documentation.md`                                                                  |
| Decision                 | completed                                                                                                  |
| Implementation reference | `docs/_resources/suggestion-driven-documentation/proposals/suggestion-driven-documentation-proposal-v1.md` |
| Created at               | 2026-06-15                                                                                                 |
| Author                   | Claude Code, claude-opus-4-8, xhigh                                                                        |
| Reviewer                 | User                                                                                                       |

## Issue

The workflow defines "Creating suggestions" (which produces documents with
`Decision: pending`) and "Applying suggestions" (which reads documents that are
already decided), but never describes the step in between: where a reviewer reads
the pending suggestions and sets `Decision` to `accepted`, `rejected`, or
`deferred`.

In practice this step happens — the existing suggestions were marked
`accepted`/`refused` before being applied — but it is undocumented. "Applying
suggestions" step 3 reads all `pending` suggestions and step 5 applies them,
which conflates "apply" with "accept" and leaves no defined place to record a
rejection or deferral.

## Suggestion

Add a short "Reviewing suggestions" section between "Creating suggestions" and
"Applying suggestions" stating that a reviewer:

- reads each `pending` suggestion and sets its `Decision` and `Reviewer`;
- promotes only `accepted` suggestions to "Applying suggestions" (which should
  then read `accepted`, not `pending`);
- records `rejected`/`deferred` outcomes in the suggestion document (and, per
  `documentation-structure.md`, links them from `INDEX.md`).
