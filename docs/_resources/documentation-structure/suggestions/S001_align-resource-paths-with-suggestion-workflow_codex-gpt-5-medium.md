# S001 - Align resource paths with suggestion workflow

| Field                    | Value                                                                                      |
|--------------------------|--------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                       |
| File                     | `docs/documentation-structure.md`                                                          |
| Decision                 | completed                                                                                  |
| Implementation reference | `docs/_resources/documentation-structure/proposals/documentation-structure-proposal-v1.md` |
| Created at               | 2026-06-15                                                                                 |
| Author                   | Codex, gpt-5, medium                                                                       |
| Reviewer                 | User                                                                                       |

## Issue

`docs/documentation-structure.md` recommends a generic sibling `_resources/`
folder next to each deliverable, while `docs/suggestion-driven-documentation.md`
uses a document-scoped `_resources/{document-name}/...` workflow for suggestions
and proposals. That leaves implementers with two plausible resource layouts for
the same document.

## Suggestion

Update the folder structure guide to use a document-scoped resource namespace,
for example:

```text
document.md
_resources/
`-- document/
    |-- INDEX.md
    |-- suggestions/
    |-- proposals/
    |-- research/
    |-- reviews/
    `-- drafts/
```

Explain that `document` is derived from the target filename without its
extension. This keeps the structure guide compatible with the suggestion-driven
workflow and avoids collisions when several documents share one folder.
