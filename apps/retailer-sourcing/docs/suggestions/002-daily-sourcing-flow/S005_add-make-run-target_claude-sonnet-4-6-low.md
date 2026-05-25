# S005 — Define `make run` target referenced in smoke test

| Field                    | Value                                                                                                                                                                                                                                 |
|--------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | low                                                                                                                                                                                                                                   |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                         |
| Decision                 | accepted                                                                                                                                                                                                                              |
| Implementation reference | `apps/retailer-sourcing/Makefile` (added `run` target invoking `cargo run --bin retailer-sourcing`); `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (file-by-file entry added for the Makefile change) |
| Created at               | 2026-05-25                                                                                                                                                                                                                            |
| Author                   | Claude Code, claude-sonnet-4-6, low                                                                                                                                                                                                   |
| Reviewer                 |                                                                                                                                                                                                                                       |

## Issue

The Bruno smoke test example references `make run`:

```bash
make up && make run    # one terminal
```

No `run` target exists in either the root `Makefile` or
`apps/retailer-sourcing/Makefile`. Running the smoke test as written would
fail immediately.

## Suggestion

Add a `run` target to `apps/retailer-sourcing/Makefile`:

```makefile
run:
	DATABASE_URL=$(DATABASE_URL) \
	RETAILER_SOURCING_BEARER_TOKEN=dev \
	cargo run --bin retailer-sourcing
```

And add it to the file-by-file section:

> `apps/retailer-sourcing/Makefile` — add `run` target that sets dev
> environment variables and starts the binary.
