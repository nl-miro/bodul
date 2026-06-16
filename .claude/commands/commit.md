---
description: Generate and create conventional commits from all changes
allowed-tools: Bash(git:*)
---

# Code Committer Agent

Analyze git changes and create well-formatted conventional commits. If changes
are logically distinct, commit them separately.

## Commit Message

Template:
```
<type>(<optional scope>): <subject>

[optional body]

[optional footer]
```

Use a body when the subject does not explain the why, impact, side effects, or
non-obvious refactor:
```
Why: <reason>
What: <change>
Tests: <commands run or why not applicable>
```

Skip the body for trivial/self-explanatory changes such as typo, formatting, or
`add login button`.

Types:
- `feat`: user-facing feature
- `fix`: user-facing bug fix
- `docs`: documentation only
- `style`: formatting/whitespace only
- `refactor`: code change without feature/bug behavior change
- `perf`: performance improvement
- `test`: tests only
- `build`: build system or dependencies
- `ci`: CI configuration/scripts
- `chore`: tooling, configs, repo hygiene, maintenance

Subject rules:
- Under 72 characters
- Imperative mood: `add caching`, not `added caching`
- Starts lowercase
- No trailing period

Other rules:
- Scope is optional unless repo guidelines require/forbid it.
- Footers may include issue refs such as `Fixes #123`, `Fixes BOD-042`, or `Relates to BOD-015, BOD-018`.
- Never add attribution/signature lines such as `Co-Authored-By:` or `Signed-off-by:`. If hooks require signatures, stop and ask.
- Never run destructive/history-rewriting commands unless explicitly requested: `git commit --amend`, `git rebase`, `git reset --hard`, `git clean -fd`.
- Never run networked git commands unless explicitly requested: `git push`, `git pull`, `git fetch`.

Examples:
```
docs: clarify install steps
fix(auth): handle expired refresh tokens
feat(api): add /health endpoint
chore: update dev dependencies

fix(payment): correct rounding in tax calculation

Fixes #1247

perf(db): add index on users.email column

Why: Email lookups in login flow were scanning full table
What: Added composite index on (email, deleted_at)
Tests: Verified query plan with EXPLAIN; benchmarks show 50x speedup
```

## Interaction Prompt

Before each logical commit, present exactly:
```
Proposed commit:
Type: <type>
Scope: <scope|none>
Subject: <subject>
Body: <none|included>
Footer: <none|included>
Linear Issue: <BOD-001|none|unknown>
Commit command:
<exact git commit command to run>
Staged files (from `git diff --staged --name-status`):
- <status> <file>
Confirm? (yes/no/edit/split/cancel)
```

Responses:
- `yes`: run the commit
- `no`: do not commit; ask what to change
- `edit`: ask for exact type/scope/subject/body/footer, then re-propose
- `split`: ask which files/hunks belong in commit A vs B, then use `git add -A -- <files>` and/or `git add -p`
- `cancel`: abort; leave staging as-is and exit

## Workflow

1. Status:
   - Run `git status --porcelain=v1 -b`; if only the `##` branch line appears, report `no changes to commit`.
   - Run `git status`; stop for merge conflicts or rebase/merge/cherry-pick/revert in progress.
2. Linear issue check:
   - Run `git branch --show-current`.
   - If the branch contains `BOD-001`-style text, extract it and offer a footer: `Relates to BOD-001`, or `Fixes`/`Closes` if the commit resolves it.
3. Inspect diffs:
   - Run `git diff` and `git diff --staged`.
   - Check unstaged and staged state before staging anything.
   - Scan for secrets/sensitive data; if found, stop and ask.
4. Group changes:
   - One group = one purpose. Prefer smaller commits when unsure.
   - Keep dependent changes together if splitting would be confusing or inconsistent.
   - Decide whether untracked files should be committed now, later, or ignored; ask if unclear.
5. Stage each group:
   - Use `git add -A -- <files>`.
   - Use `git add -p` for mixed concerns in one file.
   - Verify with `git diff --staged` and list with `git diff --staged --name-status`.
6. Commit each group:
   - Draft the header as `<type>(<scope>): <subject>` or `<type>: <subject>`.
   - Add body/footer only when useful or required.
   - Show the Interaction Prompt and wait for confirmation.
   - Commit with one `-m` per paragraph:
     - Header only: `git commit -m "<header>"`
     - Header + body: `git commit -m "<header>" -m $'<body>'`
     - Header + footer: `git commit -m "<header>" -m "<footer>"`
     - Header + body + footer: `git commit -m "<header>" -m $'<body>' -m "<footer>"`
   - Verify with `git show --name-status --oneline -1` and `git status --porcelain=v1 -b`.
7. Summary:
   ```
   Commits created:
   - <hash> <type>(<scope>): <subject>
     <status> <file>
   ```

## Command Reference

- `git status --porcelain=v1 -b`: machine-friendly branch + status codes; clean means only `## <branch>`.
- `git status`: human-readable staged, unstaged, untracked, conflict, and in-progress operation state.
- `git diff`: unstaged patch.
- `git diff --name-only`: unstaged file paths.
- `git diff --staged` / `git diff --cached`: staged patch.
- `git diff --staged --name-only`: staged file paths.
- `git diff --staged --name-status`: staged files with status codes.
- `git diff --name-only --diff-filter=U`: conflicted/unmerged files.
- `git add -A -- <files>`: stage specified additions, modifications, deletions.
- `git add -p`: interactively stage selected hunks.
- `git restore --staged -- <files>`: unstage without discarding edits.
- `git commit -m "<message>"`: commit with subject.
- `git commit -m "<message>" -m $'<body>'`: commit with multi-line body.
- `git show --name-status --oneline -1`: verify last commit summary and files.

Status codes:
- `??`: untracked
- ` M`: modified, unstaged
- `M `: modified, staged
- `A `: added, staged
- `D `: deleted, staged
- `UU`: unmerged/conflict
- `R100`: renamed

## Grouping Examples

- README update + new docs files = one `docs` commit.
- Bug fix in auth + new dashboard feature = two commits: `fix` + `feat`.
- Refactor + matching tests = one `refactor` commit.
- `.gitignore` + eslint config = two commits if they are different tooling concerns.
- Dependency manifest + lockfile = one `build` or `chore` commit.
- Formatting across files + unrelated logic fix = two commits: `style` + `fix`.
- Rename/move + mechanical import updates = one `refactor` or `chore` commit.
- Migration + app code using it = one `feat` or `fix` commit unless org policy splits migrations.
- Generated files + source changes = usually two commits unless generated artifacts are part of the same change.
- Vendor/third-party updates + local modifications = two commits.

## Troubleshooting

- Nothing to commit: if clean, report `no changes to commit`.
- Unrelated staged files: `git restore --staged -- <files>`, then re-stage intended files/hunks and re-check.
- Mixed concerns in one file: use `git add -p`.
- Hook failure: show output; do not bypass unless explicitly requested. Fix issues or reduce scope. If requested, use `git commit --no-verify -m "<header>" [-m $'<body>'] [-m "<footer>"]`.
- Lockfile issues: keep manifest + lockfile together; if conflicted, stop and ask which tool should regenerate it.
- Merge conflicts: stop committing, identify with `git diff --name-only --diff-filter=U`, resolve, stage, re-check.
- Operation in progress: stop and ask whether to continue or abort. Do not run `git rebase --continue/--abort`, `git cherry-pick --continue/--abort`, or `git revert --continue/--abort` without confirmation.
- Git lock error (`index.lock` or another process): stop and ask; do not delete lock files automatically.
- Signature required: do not add signatures; stop and ask.

## Issue/Org Conventions

- Check `CONTRIBUTING.md`, PR templates, or wiki for repo-specific commit rules.
- If required rules are unclear, ask for ticket format, allowed scopes, and type restrictions.
- GitHub/GitLab examples: `Fixes #123`, `Closes #123`, `Resolves !456`.
- Jira example: `PROJECT-456` in subject or body.
- Linear:
  - Branches often look like `nulllabs/bod-001-issue-title`; include the issue ID when committing on those branches.
  - Commits with `BOD-001` auto-link after push.
  - `Fixes`, `Closes`, or `Resolves` can move issues to Done.
  - `Relates to BOD-001` links without status change.
  - Multi-issue footer: `Fixes BOD-001, BOD-002`.
  - Prefer single-issue commits when possible.
- Default scope rule: scopes are optional.
- If org type restrictions exist, follow them before committing.

## Glossary

- Imperative mood: verb-led subject like `add`, `fix`, `remove`, `update`.
- Type: commit category used for changelogs/versioning.
- Scope: short area label in parentheses, e.g. `auth`, `api`, `cli`.
- Footer: metadata lines at the end, e.g. `Closes #123`.
- Staged changes: changes in the Git index, ready to commit.
- Unstaged changes: working tree changes not yet in the index.
- Partial staging: staging selected hunks/lines, usually with `git add -p`.

Now analyze all changes and create the appropriate commit(s).
