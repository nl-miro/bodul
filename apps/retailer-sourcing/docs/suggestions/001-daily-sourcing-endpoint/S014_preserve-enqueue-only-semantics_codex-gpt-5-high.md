# S014 - Preserve enqueue-only request semantics

| Field                    | Value                                                                                                                                                                                                                                                  |
|--------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                                                   |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                                                      |
| Decision                 | accepted                                                                                                                                                                                                                                               |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (Startup updated to use `start_persistent(pool, 0)`; enqueue-only semantics documented; verification test case 4 added to assert command stays in `Received` status) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                             |
| Author                   | Codex, gpt-5, high                                                                                                                                                                                                                                     |
| Reviewer                 |                                                                                                                                                                                                                                                        |

## Issue
The context says the HTTP request must only persist the command and must not execute it. The startup section also says to use the mulac kernel persistent startup path. In mulac, `PersistentKernelState::dispatch_command(...)` can drain command and event consumers for the configured `drain_rounds` after dispatch.

If the endpoint dispatches through `PersistentKernelState` with a non-zero drain setting, the request path may execute the command synchronously, which contradicts the plan's core separation between HTTP request and consumer execution. If the endpoint dispatches through a raw `CommandGateway` to avoid draining, that conflicts with the stated kernel-first integration model unless the plan documents that boundary.

## Suggestion
Specify how enqueue-only behavior is preserved when using the kernel persistent path.

Concrete options:

- configure the kernel used by the HTTP service with `drain_rounds = 0` and call `PersistentKernelState::dispatch_command(...)`;
- pass a dedicated enqueue-only gateway/state object into the HTTP app while the worker loop owns command execution;
- move worker startup and request-time draining into scope and explicitly change the endpoint semantics away from enqueue-only.

The plan should also add a verification case that proves the HTTP request persists a `Received` command without running the `StartDailySourcing` handler in the same request.


## Decision

command gateway handles direct execution vs enqueue-only. most of the services will be enqueue-only.