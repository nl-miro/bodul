# EDR-0004: Monorepo with Layered Architecture

* **Status:** Proposed
* **Driver:** @miro
* **Approver:** @miro
* **Date:** 2026-01-13
* **Area:** Architecture

## Context and Problem Statement

We need to define how code is organized within the repository, balancing code reusability, clear separation of concerns, and maintainability across multiple applications.

## Decision Drivers

* Working on proof of concept using multiple repos seems hard to be fast
* Maintainability and scalability of the codebase later
* Clear separation of concerns in codebase
* Ability to share code without dependency management overhead

## Considered Options

1. **Monorepo with layered architecture**: Single repository with clear separation
2. **Polyrepo**: Separate repositories per service
3. **Monolith**: Single application without library separation

## Decision Outcome

Chosen option: **Monorepo with layered architecture**

### Structure

```
/
├── lib/           # Non-product-specific libraries (reusable across projects)
├── shared/        # Shared libraries (product-specific but cross-application)
└── <app>/         # Applications at root level
    ├── domain/    # Business logic, entities, value objects
    ├── app/       # Application services, use cases
    ├── infra/     # Infrastructure implementations (DB, external services)
    ├── bootstrap/ # Application startup, dependency injection
    └── web/       # HTTP handlers, API routes
```

### Rationale

* `lib/` contains truly generic, reusable code that could be open-sourced or used in other projects
* `shared/` contains product-specific code shared between multiple applications
* Each application follows clean architecture principles with clear boundaries between layers

## Consequences

### Positive
* Clear code organization promotes maintainability
* Easy code sharing without dependency management overhead
* Single source of truth for all code
* Atomic commits across multiple packages

### Negative
* Monorepo requires tooling for efficient builds
* Initial setup complexity is higher
* Repository size grows over time

## Links

* [Monorepo Explained](https://monorepo.tools/)
