# S013 - Align with kernel command boundary

| Field                    | Value                                                                                                                                                                                                                                                                                                |
|--------------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | high                                                                                                                                                                                                                                                                                                 |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                                                                                                                                                                    |
| Decision                 | accepted                                                                                                                                                                                                                                                                                             |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (`StartDailySourcing` defined as `kernel::ApplicationCommand`; command envelope updated to `kernel::NewCommandEnvelope<StartDailySourcing>`; handler dispatches through `PersistentKernelState::dispatch_command`) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                                           |
| Author                   | Codex, gpt-5, high                                                                                                                                                                                                                                                                                   |
| Reviewer                 |                                                                                                                                                                                                                                                                                                      |

## Issue
The plan says startup should use the mulac kernel persistent path, but the endpoint design still constructs a raw `commanding::io::NewCommandEnvelope` and calls `CommandGateway::dispatch(...)` directly. That bypasses the kernel's typed application command boundary, where application commands implement `kernel::ApplicationCommand` and are dispatched through `kernel::NewCommandEnvelope<C>`.

This creates two conflicting integration models in the same plan:

- kernel-owned composition and state at startup;
- direct low-level command gateway dispatch inside the HTTP feature.

It also makes the `StartDailySourcing` command shape implicit: the plan defines a payload struct, but not the application command type that the kernel should serialize and route.

## Suggestion
Define `StartDailySourcing` as an application command type and make it implement `kernel::ApplicationCommand`. Update the command-envelope section to use `kernel::NewCommandEnvelope<StartDailySourcing>` with `kernel::NewCommandMetadata`, then dispatch through the service state boundary chosen for mulac kernel integration.

For example, the plan should specify one of these models explicitly:

- `app()` receives a service-owned state wrapper containing `kernel::PersistentKernelState`, and the handler calls `state.dispatch_command(...)`;
- `app()` receives a lower-level `Arc<CommandGateway>`, but the plan states this intentionally bypasses the typed kernel command boundary and explains why.

Prefer the first model if the architectural decision is "use mulac kernel to wire all of that up."


## Decision

use mulac kernel to wire all of that up
