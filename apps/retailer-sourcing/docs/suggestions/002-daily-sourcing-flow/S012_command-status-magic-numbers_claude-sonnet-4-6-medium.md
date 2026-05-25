# S012 — Command/event `status` values in tests are magic numbers

| Field                    | Value                                                                                                                                                                                                                                                                                 |
|--------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Priority                 | medium                                                                                                                                                                                                                                                                                |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md`                                                                                                                                                                                                         |
| Decision                 | accepted                                                                                                                                                                                                                                                                              |
| Implementation reference | `apps/retailer-sourcing/docs/implementation-plans/002-daily-sourcing-flow.md` (Verification gains a "Status values" subsection with a 0/1/2 → Received/Reserved/Processed table linked to mulac's commanding assembly; `wait_for` example now uses named `STATUS_PROCESSED` constant) |
| Created at               | 2026-05-25                                                                                                                                                                                                                                                                            |
| Author                   | Claude Code, claude-sonnet-4-6, medium                                                                                                                                                                                                                                                |
| Reviewer                 |                                                                                                                                                                                                                                                                                       |

## Issue

Verification uses bare integers for `command_entries.status` and
`event_entries.status`:

```rust
wait_for(Duration::from_secs(10), || {
    // query command_entries.status WHERE id = … returns 2 (Processed)
})
```

…and elsewhere refers to "status `Received`" without saying what integer
that is. Plan 001's verification test 4 asserted `status == 0` for
`Received`; plan 002's wait_for example uses `2` for `Processed` without
explaining the mapping. An implementer reading the plan in isolation has no
way to know whether `Received = 0` and `Processed = 2` are correct, what
`1` means, or whether the same scale applies to `event_entries`.

These numbers are determined by the `CommandStatus` and (event-side)
equivalent enums inside the mulac kernel/diesel layer. The plan currently
hides that dependency, which makes test writing brittle (an implementer
will reach for "looks like 0 worked last time, let me try 2").

## Suggestion

In the "Polling helper" subsection (or a new "Status values" sub-subsection
under Verification), document where these come from and the integer
mapping:

> `command_entries.status` and `event_entries.status` are persisted from
> mulac's `CommandStatus`/`EventStatus` enums. The integer encoding used by
> the diesel layer is:
>
> | Value | Command meaning | Event meaning |
> |-------|-----------------|---------------|
> | 0     | Received        | Received      |
> | 1     | Reserved        | Reserved      |
> | 2     | Processed       | Processed     |
>
> See
> [libs/commanding/src/assembly/](https://github.com/nulllabsdev/mulac/tree/main/libs/commanding/src/assembly)
> for the source of truth.

Then update the wait_for example to use a clearly-named local constant:

```rust
const STATUS_PROCESSED: i32 = 2;

wait_for(Duration::from_secs(10), || {
    let status: i32 = /* query command_entries.status WHERE id = … */;
    status == STATUS_PROCESSED
})
.await;
```

This costs almost nothing in plan length and removes a real implementation
trap.
