# EDR-0003: PostgreSQL as Primary Database

* **Status:** Accepted
* **Driver:** @miro
* **Approver:** @miro
* **Date:** 2026-01-13
* **Area:** Architecture

## Context and Problem Statement

We need to choose a primary database for persistent data storage that provides reliability, performance, and rich feature support for our application needs.

## Decision Drivers

* Geospatial Support: Includes PostGIS extension for advanced handling of geospatial data
* Full-Text Search: Built-in capabilities for powerful full-text search functions
* High Concurrency: Utilizes Multiversion Concurrency Control (MVCC) to manage high levels of concurrent operations without locking
* Replication and Clustering: Supports various replication methods and clustering solutions for improved availability and scalability

## Considered Options

1. **PostgreSQL**: Mature, feature-rich, excellent for relational data
2. **MySQL**: Widely used, good performance
3. **MongoDB**: Flexible schema, document-based

## Decision Outcome

Chosen option: **PostgreSQL**

### Rationale

PostgreSQL is a battle-tested relational database with excellent support for complex queries, ACID compliance, and a rich feature set including JSONB for semi-structured data.

## Consequences

### Positive
* ACID compliance ensures data integrity
* Rich extension ecosystem (PostGIS, pg_trgm, etc.)
* Excellent JSON/JSONB support for semi-structured data
* Strong community and long-term support

### Negative
* Lack of contacts that have experience with clustering and replication

## Links

* [PostgreSQL](https://www.postgresql.org/)
* [PostGIS](https://postgis.net/)
