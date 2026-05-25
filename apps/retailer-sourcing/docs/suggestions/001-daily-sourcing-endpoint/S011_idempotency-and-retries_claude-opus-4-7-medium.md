# S011 - Server-generated command_id blocks safe client retries

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | medium                                                                            |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | deffered                                                                          |
| Implementation reference |                                                                                   |
| Created at               | 2026-05-25                                                                        |
| Author                   | Claude Code, claude-opus-4-7, medium                                              |
| Reviewer                 |                                                                                   |

## Issue

The handler generates a fresh `Uuid::new_v4()` per request and persists it as
the command's `command_id`. The context section frames this as a feature:
"each request records a fresh command entry."

The problem: the only realistic callers are an operator (curl/Bruno) or a
scheduler (cron, k8s CronJob, GitHub Actions). All of these can fail mid-call
— TCP timeout, connection reset, 5xx with a recorded write. The caller has no
way to know whether the command was persisted or not. The only safe behavior
is to retry. Each retry produces a new `StartDailySourcing` command and a
duplicate downstream sourcing run.

For a daily job, that may be acceptable today (run twice). But the
HTTP-to-command-queue path is intentionally a contract surface for other
services; the next caller may not tolerate duplicates (e.g., a per-retailer
trigger).

The deeper issue is that the current design conflates two distinct concerns:

- **Command identity** — globally unique, owned by whoever is responsible
  for the operation. Should be assignable by the caller.
- **Request identity** — per-HTTP-request, owned by the server. Can be a
  fresh UUID per call.

`NewCommandMetadata.command_id` is documented in mulac as "the caller must
supply `command_id`; no ID generation occurs inside the system boundary."
The plan is currently inserting an HTTP boundary that violates that
contract.

## Suggestion

Pick one of these explicitly and document the tradeoff:

1. **Accept `Idempotency-Key` header.** If supplied, use it as the
   `command_id` (after parsing as UUID); if not, generate one. Returns
   the same `command_id` on duplicate requests, and the commanding store's
   uniqueness constraint dedupes naturally.
2. **Accept a JSON body with `command_id`.** Slightly more rigid; ties the
   endpoint to the existing `NewCommandEnvelope` shape.
3. **Document that this endpoint is for at-most-once human/scheduled use
   only**, and explicitly call out that retries cause duplicate sourcing
   runs. Acceptable if the downstream consumer is idempotent on the
   payload.

Recommended: option (1). It's the standard HTTP idempotency pattern, doesn't
require a body, and matches mulac's contract about caller-supplied
`command_id`. Add a Bruno test case that sends the same idempotency key
twice and asserts the same `command_id` comes back.

If option (3) is chosen, make that explicit in the Context section so the
downstream consumer plan knows it has to handle deduplication.
