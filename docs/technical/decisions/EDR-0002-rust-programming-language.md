# EDR-0002: Programming Languages

* **Status:** Proposed
* **Driver:** @miro
* **Approver:** @miro
* **Date:** 2026-01-13
* **Area:** Architecture

## Context and Problem Statement

We need to choose a primary programming language for backend services that provides safety, performance, and maintainability for long-term development.

## Decision Drivers

* Language safety: strong type system, strict compile time checks, memory safety, high performance and enforcing thread-safe patterns
* Single-file deployment: Static linking produces self-contained binaries, simplifying distribution, container images, and cold-starts
* Explicit types and compile-time guarantees make refactoring and code reviews safer and faster

## Considered Options

1. **Rust**: High performance, memory safety, strong type system
2. **Go**: Simple, fast compilation, good for services
3. **TypeScript/Node.js**: Large ecosystem, familiar to many developers

## Decision Outcome

* **Primary language:** Rust
* **Secondary language:** Go

### Rationale

**Rust** is the primary language for production backend services. It provides memory safety without garbage collection, excellent performance, and a strong type system that catches errors at compile time.

**Go** is used as a secondary language for:
* Proof of concepts and prototypes
* Instrumentation and tooling
* Quick throwaway code and scripts

Go's simplicity and fast compilation make it ideal for rapid iteration when the safety guarantees of Rust are not critical.

## Consequences

### Positive
* Strong type safety prevents bugs at compile time
* High performance comparable to C/C++
* Memory safety without garbage collection pauses
* Excellent tooling (Cargo, clippy, rustfmt)

### Negative
* Steeper learning curve compared to Go or TypeScript
* Longer compilation times
* Smaller talent pool

## Links

* [Rust Programming Language](https://www.rust-lang.org/)
* [Go Programming Language](https://go.dev/)
