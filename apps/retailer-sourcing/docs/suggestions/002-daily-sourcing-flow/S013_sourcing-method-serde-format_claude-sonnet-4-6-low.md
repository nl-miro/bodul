# S013 — `SourcingMethod` JSON wire format is not specified

| Field                    | Value                                                                                                                                                                                                                                                                  |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | low                                                                                                                                                                                                                                                                    |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                          |
| Decision                 | accepted                                                                                                                                                                                                                                                               |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (`SourcingMethod` derive now includes `PartialEq, Eq` and `#[serde(rename_all = "PascalCase")]`; inline comment documents the wire format and that variant renames are breaking changes) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                             |
| Author                   | Claude Code, claude-sonnet-4-6, low                                                                                                                                                                                                                                    |
| Reviewer                 |                                                                                                                                                                                                                                                                        |

## Issue

`SourcingMethod` is shown with a plain default derive:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourcingMethod {
    WebScraping,
    Sitemap,
}
```

With the default derive, JSON encodes variants as bare strings:
`"WebScraping"` and `"Sitemap"`. That happens to be fine for now, but the
plan deliberately specifies the wire format for `RetailerCode` (see S006:
PascalCase via `#[serde(into = "String", try_from = "String")]`). Leaving
`SourcingMethod`'s wire format implicit creates an inconsistency: one field
on `SourceRetailer` has a documented JSON contract, the sibling field does
not.

It also opens the door to a future "I'll just rename the variant" change
that silently breaks payloads already persisted in `command_entries`.

## Suggestion

Either accept the default representation and document it explicitly, or
pin it the same way `RetailerCode` is pinned. Cheapest option:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SourcingMethod {
    WebScraping,
    Sitemap,
}
```

`rename_all = "PascalCase"` is a no-op for the current variants (they're
already PascalCase) but locks the wire format in even if a future variant
is added as `WebsiteRss` or similar.

Add a short note next to the enum definition in the plan:

> `SourcingMethod` serializes as `"WebScraping"` / `"Sitemap"` and is
> stored verbatim in `SourceRetailer.payload`. Renaming a variant is a
> wire-format breaking change.
