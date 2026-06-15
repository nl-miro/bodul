# S002 - Reconcile decision state vocabulary

| Field                    | Value                                      |
|--------------------------|--------------------------------------------|
| Priority                 | high                                       |
| File                     | `docs/suggestion-driven-documentation.md`  |
| Decision                 | pending                                    |
| Implementation reference |                                            |
| Created at               | 2026-06-15                                 |
| Author                   | Claude Code, claude-opus-4-8, xhigh        |
| Reviewer                 |                                            |

## Issue

The allowed values for the `Decision` field disagree with the surrounding prose,
and real usage has already drifted from the spec:

- Line 50 lists the allowed states as `pending`, `approved`, `accepted`,
  `rejected`, `deferred`.
- Line 54 (Reviewer guidance) says to populate `Reviewer` when `Decision` is
  `completed`, `rejected`, or `deferred` — but `completed` is not in the allowed
  set.
- `approved` and `accepted` are near-synonyms with no defined distinction, so it
  is unclear which to use.
- The existing `documentation-structure` suggestion files already use `accepted`
  and `refused`; `refused` is not in the allowed set either (the set has
  `rejected`).

## Suggestion

Define a single canonical state set and use it everywhere:

- Pick one lifecycle, e.g. `pending` → `accepted` | `rejected` | `deferred`,
  with one optional terminal state (`completed`) to mark "accepted and applied"
  if that distinction is needed.
- Drop the redundant `approved`/`accepted` pair (keep one).
- Standardize on `rejected` (not `refused`).
- Update line 54 to reference only states that exist in the set.
- Align the existing suggestion files, or note explicitly that legacy `refused`
  maps to `rejected`.
