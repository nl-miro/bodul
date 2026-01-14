# EDR-0006: Internal Service Security

* **Status:** Proposed
* **Driver:** @miro
* **Approver:** @miro
* **Date:** 2026-01-14
* **Area:** Security

## Context and Problem Statement

We need to establish good security practices from the start. As the platform grows with multiple services (Retriever, Crawler, Scraper, API), secure inter-service communication is critical. Retrofitting security later is costly and error-prone.

## Decision Drivers

- Minimal infrastructure for early phase
- Speed of implementation
- Security from day one without over-engineering
- Clear upgrade path as platform scales

## Considered Options

### Option 1: Auth service + client with database

Centralized authentication service storing credentials in database.

**Pros:**
- Credentials can be rotated without redeployment
- Single source of truth for all service credentials
- Supports dynamic service registration

**Cons:**
- Additional infrastructure (database, service)
- Network dependency for auth checks
- More complex to set up initially
- Database becomes a critical dependency

### Option 2: Auth service + client with hardcoded data

Centralized authentication service with credentials hardcoded in config.

**Pros:**
- Simpler than database option
- Single point of validation
- No database dependency

**Cons:**
- Credential rotation requires service redeployment
- Still requires running an additional service
- Network dependency for auth checks

### Option 3: Auth client only with hardcoded data

No service; each client validates credentials locally against hardcoded values.

**Pros:**
- Simplest to implement
- No network dependency for validation
- No additional service to run
- Fast validation (local)

**Cons:**
- Credential rotation requires redeploying all services
- Credentials duplicated across services

## Decision Outcome

Chosen option: **Option 3 — Auth client only with hardcoded data**

For the initial phase, each service validates credentials locally against hardcoded values. This minimizes infrastructure complexity while establishing the authentication pattern. Can upgrade to a centralized auth service (Option 1 or 2) as the platform scales.

## Implementation

The `auth_client` library in `shared/auth_client` implements this decision:

- **Bearer token validation**: Services validate `Authorization: Bearer <token>` headers
- **Type-safe identifiers**: `ClientId`, `Permission`, and `Role` enums for compile-time safety
- **Hardcoded credentials**: Token-to-identity mapping defined as const in the library
- **ServiceIdentity**: Validation returns client ID, permissions, and roles

Services integrate by:
1. Adding `auth_client` as a dependency
2. Creating an `AuthClient` instance
3. Calling `validate(auth_header)` or `validate_token(token)`

## Consequences

### Positive

- Fast to implement, no new infrastructure
- No network dependency for auth validation
- Establishes authentication pattern for future upgrades

### Negative

- Credential rotation requires redeploying all services
- Credentials duplicated across services

## Links

- [TS-0002: Retriever Service](../specifications/TS-0002-retriever-service.md) — first service using this auth pattern
- [auth_client library](../../../shared/auth_client) — implementation of this decision

