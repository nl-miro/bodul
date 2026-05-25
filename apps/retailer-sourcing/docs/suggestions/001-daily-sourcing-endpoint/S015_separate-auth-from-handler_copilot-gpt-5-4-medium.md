# S015 - Separate auth from handler

| Field                    | Value                                                                                                                                                                                                                                                    |
|--------------------------|----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                   |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                                                        |
| Decision                 | accepted                                                                                                                                                                                                                                                 |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (`trigger` description updated: handler receives already-authenticated request and does not parse `Authorization` header; auth fully owned by `BearerAuth` middleware) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                               |
| Author                   | GitHub Copilot, gpt-5.4, medium                                                                                                                                                                                                                          |
| Reviewer                 |                                                                                                                                                                                                                                                          |

## Issue

The feature-module section says `trigger(...)` "extracts the bearer token"
before dispatching the command, but the plan also defines a reusable
`BearerAuth` middleware that authenticates the route before the handler runs.
That creates two overlapping authentication responsibilities in the same
endpoint design.

If implemented literally, the handler may duplicate header parsing, drift from
the middleware's validation rules, or accidentally reintroduce auth bugs when
the middleware is reused elsewhere. It also makes the handler less reusable and
harder to test because it now owns both transport auth and command dispatch.

## Suggestion

Make the handler auth-agnostic and let the middleware own bearer-token parsing
and validation completely.

Update the private-items bullet for `trigger(...)` to say it:

- receives an already-authenticated request;
- builds a fresh command envelope;
- dispatches it;
- returns `202 Accepted` with the response DTO.

If the handler eventually needs caller identity, document that the middleware
inserts a typed auth context into request extensions rather than requiring the
handler to parse the `Authorization` header again.
