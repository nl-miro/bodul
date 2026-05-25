# S012 - HARDCODED_BEARER_TOKEN env var name is misleading

| Field                    | Value                                                                                                                                                                                                                |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | low                                                                                                                                                                                                                  |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                    |
| Decision                 | accepted                                                                                                                                                                                                             |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (Renamed `HARDCODED_BEARER_TOKEN` to `RETAILER_SOURCING_BEARER_TOKEN` throughout; added inline comment explaining the auth scheme) |
| Created at               | 2026-05-25                                                                                                                                                                                                           |
| Author                   | Claude Code, claude-opus-4-7, low                                                                                                                                                                                    |
| Reviewer                 |                                                                                                                                                                                                                      |

## Issue

The env var `HARDCODED_BEARER_TOKEN` reads as a placeholder name — as if the
value itself is hardcoded somewhere, or as if "HARDCODED" describes the
literal token rather than the auth scheme. In production deployments, this
variable will be set from a secret (Kubernetes secret, Vault, etc.) — the
value is not hardcoded; the *mechanism* (single shared static token) is
hardcoded vs. a more dynamic auth scheme (OAuth, mTLS, signed JWTs).

This name will confuse:

- New operators reading deployment manifests ("why is our secret called
  hardcoded?").
- Future readers looking for "the real" auth config ("is there a
  non-hardcoded variant somewhere?").
- Anyone grep'ing for `BEARER_TOKEN` or `AUTH_TOKEN` standard conventions.

The "hardcoded" framing belongs in code comments explaining the auth
*scheme*, not in the variable name.

## Suggestion

Rename to one of:

- `RETAILER_SOURCING_BEARER_TOKEN` — service-scoped, follows the
  `{SERVICE}_{CONCERN}_{ATTR}` convention some shops use.
- `BEARER_TOKEN` — simple, fine for a single-service deployment.
- `API_BEARER_TOKEN` — generic enough to cover all future authenticated
  endpoints on this service.

Add a comment near the env-var read site explaining the scheme:

```rust
// Single static shared-secret auth — adequate for internal-only use.
// Revisit when more callers need distinct credentials (see open questions).
let token = std::env::var("RETAILER_SOURCING_BEARER_TOKEN")?;
```

Update the configuration table and all in-document references in one pass.

## Decision

Use RETAILER_SOURCING_BEARER_TOKEN