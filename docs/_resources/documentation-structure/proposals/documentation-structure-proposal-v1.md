# Documentation Structure Guide
## Keeping Documents Clean While Preserving the Research Trail

---

## The Core Problem

You want the **document itself to stay readable**, while keeping a full **audit trail of resources** accessible but out of the way.

The risk: mixing resources into the document creates confusion for readers. But discarding them loses the "road travelled" — valuable for investigating and improving workflows later.

**Solution: two-layer approach.**

---

## Recommended Folder Structure

```
/project-name/
│
├── document.md              ← The actual deliverable (clean, readable)
│
└── _resources/              ← Everything that fed into it
    └── document/            ← Document-scoped namespace (one per deliverable)
        ├── INDEX.md         ← Master log: what was used, when, why
        ├── suggestions/     ← Actionable review feedback
        │   ├── S001_....md
        │   └── S002_....md
        ├── proposals/       ← Working copies updated before the deliverable
        │   ├── document-proposal-v1.md
        │   └── document-proposal-v2.md
        ├── research/
        │   ├── source-1.md
        │   └── source-2.pdf
        ├── reviews/
        │   ├── review-round-1.md
        │   └── feedback-alice.md
        └── drafts/
            ├── draft-v1.md
            └── draft-v2.md
```

The `_resources/` prefix signals "supporting material" visually — present but
clearly separate from the deliverable. Within it, each deliverable gets its own
namespace folder named after the target filename **without its extension** (so
`document.md` → `_resources/document/`). This keeps the structure compatible
with the suggestion-driven workflow and avoids collisions when several documents
share one `_resources/` parent.

---

## The Key: `INDEX.md` as the Bridge

This file solves the confusion problem. It's a lightweight log that *links* the document to its resources without cluttering the document itself.

```markdown
# Resource Index — [Document Name]

## Sources Used
| Resource          | Type     | Used For         | Section in Doc |
|-------------------|----------|------------------|----------------|
| source-1.md       | Research | Background stats | Introduction   |
| feedback-alice.md | Review   | Restructured §3  | Methods        |

## Decision Log
- Dropped source-2 — outdated (2019), superseded by source-1
- Alice's review prompted merging sections 3 and 4

## Open Questions
- [ ] Verify stat in §2 against primary source
```

The decision log is especially valuable for workflow review — it captures *why* things were included or dropped, not just *what*.

---

## Suggestion and Proposal Workflow

Review feedback never edits the deliverable directly. It flows through the
document-scoped namespace before reaching the reader-facing file:

- **Suggestions** — actionable review feedback goes into
  `_resources/{document-name}/suggestions/`, one document per suggestion.
- **Proposals** — proposal copies of the deliverable go into
  `_resources/{document-name}/proposals/`, where suggestions are applied and
  reviewed first.
- **Controlled updates** — the deliverable is updated only after pending
  suggestions are accepted, refused, or deferred; an accepted suggestion is
  applied to a proposal copy, then the proposal is copied back over the
  original.
- **Traceability** — each suggestion document is the source of truth for its
  own decision. `INDEX.md` links to or summarizes accepted, refused, and
  deferred suggestions without restating their status.

This gives reviewers and LLM tools a clear path from feedback to controlled
document updates. See `suggestion-driven-documentation.md` for the full
lifecycle and file-naming rules.

---

## Principles

| Principle              | How it's achieved                          |
|------------------------|--------------------------------------------|
| Document stays clean   | Readers never see the scaffolding          |
| Trail is explicit      | INDEX links decisions, not just files      |
| Investigation-friendly | Future you can reconstruct every choice    |
| Scalable               | Works for solo work and team collaboration |

---

## Lighter Alternative (Single-File Documents)

If you're working in a single document rather than a folder, add a collapsible appendix at the end:

```markdown
# My Document

[...main content...]

---

## Appendix: Research Trail
*Not part of the deliverable — for workflow review only*

### Sources
- ...

### Review Notes
- ...
```

This keeps everything in one file while visually separating the deliverable from the supporting material.

---

## Summary

> **The document answers *what*. The resource trail answers *how you got there*.**

Valuable for workflow improvement — but only surfaced when you need it.
