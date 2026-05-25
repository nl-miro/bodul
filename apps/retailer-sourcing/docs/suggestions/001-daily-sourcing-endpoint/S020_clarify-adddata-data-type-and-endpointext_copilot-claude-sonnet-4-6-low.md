# S020 - Clarify AddData/Data<T> type and required EndpointExt import

| Field                    | Value                                                                             |
|--------------------------|-----------------------------------------------------------------------------------|
| Priority                 | low                                                                               |
| File                     | `apps/retailer-sourcing/docs/implementation-plans/001-daily-sourcing-endpoint.md` |
| Decision                 | accepted                                                                          |
| Implementation reference | commit 013880c                                                                    |
| Created at               | 2026-05-25                                                                        |
| Author                   | GitHub Copilot, claude-sonnet-4.6, low                                            |
| Reviewer                 | GitHub Copilot, claude-sonnet-4.6                                                 |

## Issue

Two Poem API details in the plan cause compile errors that are not obvious from
reading documentation or the plan text alone:

**1. `Data<&Arc<T>>` vs `Data<&T>`**
The plan says the handler receives `Data(&Arc<PersistentKernelState>)`. In reality,
`AddData::new(val: T)` wraps `T` in an `Arc<T>` _internally_. The handler extractor
is `Data<&T>`, not `Data<&Arc<T>>`. Writing `Data<&Arc<PersistentKernelState>>`
compiles only if the route was registered with `AddData::new(arc_val)` where
`arc_val: Arc<PersistentKernelState>` (producing `Arc<Arc<PersistentKernelState>>`
internally) — which is awkward and should be avoided.

The correct pattern:
```rust
// register():  AddData::new((*state).clone())   // state: Arc<KernelState> → unwrap to KernelState
// handler:     state: Data<&PersistentKernelState>   // extracts &KernelState from inner Arc
let state = state.0.clone();                     // PersistentKernelState: Clone
```

**2. `EndpointExt` must be in scope for `.with()`**
`post(trigger).with(AddData::new(...)).with(BearerAuth::new(...))` fails to compile
with "no method named `with`" unless `poem::EndpointExt` is imported. The plan shows
the chained `.with()` pattern without mentioning this import requirement. The compiler
error message is clear, but it adds a lookup round-trip during implementation.

## Suggestion

In the "Feature module" design section, add a callout:

> Note: `.with(...)` is a method on `poem::EndpointExt`. Import it explicitly:
> `use poem::EndpointExt;`. `AddData::new(val: T)` stores `Arc<T>` internally;
> the handler extracts `Data<&T>`, not `Data<&Arc<T>>`.

Update the handler parameter type in the code sketch to `Data<&PersistentKernelState>`.
