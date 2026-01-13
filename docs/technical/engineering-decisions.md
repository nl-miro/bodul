# Engineering Decisions

A lightweight record of technical / engineering decisions.

## How to use
- Create a new memo for decisions that are **hard to reverse**, **cross-team**, or **will matter in 3+ months**.
- Keep entries short. The memo holds the details.
- Never delete decisions. If a decision changes, mark it **Superseded** and add a new decision.

## Status meanings
- **Proposed**: under discussion, not final
- **Accepted**: decided and active
- **Deprecated**: still true, but no longer recommended for new work
- **Superseded**: replaced by another decision (link to the new one)
- **Rejected**: explicitly decided not to do

## Decisions

|   ID | Date       | Area          | Title                              | Status   | Driver | Approver | Link                                                                 | One-line outcome                                                   |
|-----:|:-----------|:--------------|:-----------------------------------|:---------|:-------|:---------|:---------------------------------------------------------------------|:-------------------------------------------------------------------|
| 0001 | 2026-01-12 | Documentation | Using linear for issue tracking    | Accepted | @miro  | @miro    | docs/technical/decisions/EDR-0001-using-linear-for-issue-tracking.md | We will be using publicly accessible linear.app for issue tracking |
| 0002 | 2026-01-13 | Architecture  | Programming languages              | Accepted | @miro  | @miro    | docs/technical/decisions/EDR-0002-rust-programming-language.md       | Rust primary, Go secondary for prototypes and tooling              |
| 0003 | 2026-01-13 | Architecture  | PostgreSQL as primary database     | Accepted | @miro  | @miro    | docs/technical/decisions/EDR-0003-postgresql-data-storage.md         | PostgreSQL for ACID compliance, PostGIS, and full-text search      |
| 0004 | 2026-01-13 | Architecture  | Monorepo with layered architecture | Accepted | @miro  | @miro    | docs/technical/decisions/EDR-0004-monorepo-architecture.md           | Monorepo with lib/, shared/, and layered app structure             |
