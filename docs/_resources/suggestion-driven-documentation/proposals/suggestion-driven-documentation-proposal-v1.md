# Suggestion-Driven Documentation

| Field   | Value      |
|---------|------------|
| Version | 0.1        |
| Date    | 2026-04-12 |

Instead of making documentation changes on the fly during a pull request review,
ask LLM reviewers to write each actionable suggestion down in a suggestion
document. Review those documents afterward, decide which suggestions to
implement, and keep them as the record of what was proposed, completed, refused,
or deferred.

The default is one suggestion per document. A single document may contain
multiple suggestions only for small editorial-only batches such as typos,
formatting fixes, or similar non-behavioral wording changes.

Store suggestions in `_resources/{document-name}/suggestions/`, where
`{document-name}` is the target file's name without its extension or directory
path.

Use the following filename format:

`S{number}_{title}_{tool-model-effort}.md`

Example: `S001_improve-heading-clarity_claude-sonnet-4-6-high.md`

- `number` must be sequential and zero-padded to three digits: `001`, `002`,
  `003`, and so on.
- Suggestion numbers are tracked per target file suggestion folder. For example,
  suggestions for `docs/contracts.md` go into
  `docs/_resources/contracts/suggestions/`.
- Suggestion numbers are never reused within the same target file suggestion
  folder. Each new suggestion gets the next available number in that folder, and
  gaps from refused or removed suggestions are acceptable.
- `title` must be a short lowercase kebab-case summary, for example
  `improve-heading-clarity`.
- `tool-model-effort` must be a lowercase hyphenated identifier for the
  authoring tool, model, and effort level, for example
  `claude-sonnet-4-6-high` or `codex-gpt-5-medium`.

Each suggestion document should use this structure:

```md
# S012 - Improve heading clarity

| Field                    | Value                                |
|--------------------------|--------------------------------------|
| Priority                 | medium                               |
| File                     | `docs/example.md`                    |
| Decision                 | pending                              |
| Implementation reference |                                      |
| Created at               | 2026-04-12                           |
| Author                   | Claude Code, claude-sonnet-4-6, high |
| Reviewer                 |                                      |

## Issue
Briefly describe the problem being pointed out.

## Suggestion
Describe the proposed change clearly and concretely.
```

- `Priority` records importance, such as `low`, `medium`, or `high`.
- `File` records the original file being reviewed, not the proposal copy.
- `Decision` records the current state of the suggestion: `pending`, `accepted`,
  `completed`, `refused`, or `deferred`.
- `Implementation reference` records the proposal artifact where the accepted
  suggestion was applied, such as
  `_resources/{document-name}/proposals/{document-name}-proposal-v1.md`.
- `Created at` records when the suggestion was written.
- `Author` records who created the suggestion. For tools, include the tool name,
  model, and effort level.
- `Reviewer` records who reviewed the suggestion. Leave it blank while
  `Decision` is `pending`. Populate it when `Decision` becomes `accepted`,
  `completed`, `refused`, or `deferred`.

## Creating suggestions

When an LLM tool is given a file to review:

1. Review the target document and write suggestions into
   `_resources/{document-name}/suggestions/`.
2. Commit the suggestion documents.

## Reviewing suggestions

Before applying suggestions, a reviewer:

1. Reads each suggestion with `Decision` set to `pending`.
2. Sets `Decision` to `accepted`, `refused`, or `deferred`.
3. Populates `Reviewer`.
4. Leaves `Implementation reference` blank until an accepted suggestion is
   applied to a proposal artifact.

Only accepted suggestions move into the applying workflow.

## Applying suggestions

1. Create a copy in `_resources/{document-name}/proposals/` named
   `{document-name}-proposal-v{number}.{extension}`. Version numbers start at
   `1`. If proposal versions already exist for that file, use the next number
   after the highest existing version.
2. Commit the proposal copy as-is.
3. Read all suggestion documents with `Decision` set to `accepted`.
4. If there are any questions for the engineering director, ask them.
5. For each accepted suggestion, apply the change to the proposal copy, set
   `Implementation reference` to the proposal artifact, and set `Decision` to
   `completed`.
6. Once every accepted suggestion targeting a proposal has been resolved, copy
   the proposal over the original file.
