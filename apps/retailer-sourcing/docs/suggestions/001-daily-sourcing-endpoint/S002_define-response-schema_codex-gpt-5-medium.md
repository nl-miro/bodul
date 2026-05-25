# S002 - Define response schema

| Field                    | Value                                                                                                                                                        |
|--------------------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                       |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md`                                                                            |
| Decision                 | accepted                                                                                                                                                     |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` (Response schema section added; handler returns DailySourcingResponse DTO) |
| Created at               | 2026-05-25                                                                                                                                                   |
| Author                   | Codex, gpt-5, medium                                                                                                                                         |
| Reviewer                 | Copilot (staff review)                                                                                                                                       |

## Issue
The endpoint contract says the response should return `202 Accepted` with a `command_id`, but the plan does not define a response DTO or wire format. That makes the API shape and test assertions implicit instead of explicit.

## Suggestion
Add a dedicated response type in the feature `io` surface, such as `DailySourcingResponse { command_id: Uuid }`, and document that the handler returns `202 Accepted` with a JSON body matching that schema. Update the verification section to assert against that explicit response shape.
