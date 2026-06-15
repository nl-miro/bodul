# S001 - Fix suggestion path examples

| Field                    | Value                                                                                                      |
|--------------------------|------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                       |
| File                     | `docs/suggestion-driven-documentation.md`                                                                  |
| Decision                 | completed                                                                                                  |
| Implementation reference | `docs/_resources/suggestion-driven-documentation/proposals/suggestion-driven-documentation-proposal-v1.md` |
| Created at               | 2026-06-15                                                                                                 |
| Author                   | Claude Code, claude-opus-4-8, xhigh                                                                        |
| Reviewer                 | User                                                                                                       |

## Issue

The canonical location for suggestions is `_resources/{document-name}/suggestions/`
(stated on line 12 and used in practice, e.g.
`docs/_resources/documentation-structure/suggestions/S001_...md`). Several
examples contradict that canonical form:

- Line 21 cites the suggestion folders as `docs/suggestions/contracts/` and
  `docs/suggestions/components/`. These drop the `_resources/` segment and invert
  the nesting (folder name `suggestions/` appears before the document name).
- Step 1 of "Creating suggestions" (line 60) writes suggestions into
  `_resources/{document-name}/suggestions/{filename}` — an extra `{filename}`
  subfolder that appears nowhere else and is not used in practice. Its
  parenthetical example again uses `docs/suggestions/contracts/`.
- `{document-name}` is used as a placeholder but is never explicitly defined,
  while `{filename}` is defined on line 60 as the reviewed file's name without
  its extension. The two placeholders refer to the same value.

## Suggestion

- Define one placeholder once (e.g. `{document-name}` = the target file's name
  without its extension or directory path) and use it consistently.
- Correct every example to the canonical form
  `docs/_resources/{document-name}/suggestions/` — for instance, suggestions for
  `docs/contracts.md` go into `docs/_resources/contracts/suggestions/`.
- Remove the extra `{filename}` level from step 1 so it reads
  `_resources/{document-name}/suggestions/`.
- Fix the line 21 examples to match.
