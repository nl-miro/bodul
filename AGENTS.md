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

## Branching

- `main` is the default branch. Fixes to `main` can be sent directly there.
- Feature work lands on `prompt{N}-dev` branches. PRs target the matching
  `prompt{N}-dev` branch; once the feature is done, that branch is merged into
  `main`.
- Day-to-day work can happen on any branch based on `dev`. Keep `dev` rebased
  against the current `prompt{N}-dev` branch, including any temporary commits
  needed during development.
