# Agent Instructions

Shared instructions for LLM coding tools (Claude Code, Codex, Cursor, etc.).
`CLAUDE.md` is a symlink to this file.

## Project

See [README.md](README.md) for the Phase 1 product discovery pipeline overview.

Bodul scrapes multiple Minisforum Shopify storefronts via two paths:
- Sitemap-based discovery (`sitemap.xml` → catalog/product pages)
- Menu-based web scraping fallback when sitemap coverage is incomplete

## Conventions

- Commit style: conventional commits (`feat:`, `fix:`, `docs:`, `chore:`).

### ASCII Flowcharts

Use ASCII flowcharts only inside fenced `text` blocks.

Box rules:
- Use standard boxes with exactly 30 inner characters:
  `+------------------------------+`.
- Center every box within columns 0-80.
- Center text inside each box.
- Keep labels short; wrap long labels across multiple centered lines.

Flow rules:
- Prefer top-to-bottom flow.
- Keep the main path centered.
- Center branch and merge connectors on the boxes they connect.
- Label decision branches with clear terms such as `yes`/`no`,
  `valid`/`invalid`, or `success`/`fail`.
- Avoid crossing lines. Split large charts instead of forcing everything into
  one diagram.

## Branching

- `main` is the default branch. Fixes to `main` can be sent directly there.
- Feature work lands on `prompt{N}-dev` branches. PRs target the matching
  `prompt{N}-dev` branch; once the feature is done, that branch is merged into
  `main`.
- Day-to-day work can happen on any branch based on `dev`. Keep `dev` rebased
  against the current `prompt{N}-dev` branch, including any temporary commits
  needed during development.
