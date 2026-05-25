# S002 - Error handling strategy clarification

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | medium                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Created at               | 2026-05-25                                                                        |
| Author                   | Claude Code, claude-haiku-4-5, medium                                             |
| Reviewer                 |                                                                                   |

## Issue

The "Command envelope" section shows `.map_err(internal_error)?` but doesn't
define `internal_error`. The plan also doesn't specify how to handle Diesel
errors, JSON serialization failures, or UUID generation (though the latter
shouldn't fail). This leaves the implementer guessing about error response
shapes and status codes.

## Suggestion

Add a brief error-handling section defining:

1. **CommandError handling:** Return `500 Internal Server Error` with a
   structured JSON body (e.g. `{ "error": "command queue unavailable" }`).

2. **JSON serialization failures:** These indicate a bug (malformed payload
   struct), so return `500 Internal Server Error`.

3. **Response shape:** Keep it minimal — Poem's `Error::from_status` is
   sufficient for auth failures (401), and a custom error type wrapping
   Poem's error enum is sufficient for server errors.

This ensures consistent error behavior across the endpoint and prevents ad-hoc
error handling.
