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

- Use fenced `text` blocks for ASCII flowcharts.
- Always use a 30-character inner width for standard boxes.
- Center the flowchart within columns 0-80 so it reads well in terminals,
  Markdown previews, and code review panes.
- Keep all boxes centered within the 0-80 span of the flowchart block.
- Center the text inside each box.
- Center each line in the flowchart block so the diagram itself sits balanced
  across the page.
- Center branch connectors and merge lines on the same axis as the boxes they
  connect.
- Keep the flow direction consistent. Prefer top-to-bottom for most docs.
- Use one box style and one arrow style throughout a chart.
- Keep box labels short. Prefer a few words that name the action or state.
- Align boxes, connectors, and labels carefully in a monospace layout.
- Mark decision nodes clearly, preferably with a question-style label.
- Label branches with terms such as `yes`/`no`, `valid`/`invalid`, or
  `success`/`fail`.
- Avoid crossing lines. Split a chart or repeat a node when crossings would make
  the flow hard to read.
- Keep the happy path obvious, usually straight down the center.
- Keep each chart focused. Split flows that grow beyond roughly 10 boxes.
- Avoid decoration that does not clarify the flow.

## Branching

- `main` is the default branch. Fixes to `main` can be sent directly there.
- Feature work lands on `prompt{N}-dev` branches. PRs target the matching
  `prompt{N}-dev` branch; once the feature is done, that branch is merged into
  `main`.
- Day-to-day work can happen on any branch based on `dev`. Keep `dev` rebased
  against the current `prompt{N}-dev` branch, including any temporary commits
  needed during development.
