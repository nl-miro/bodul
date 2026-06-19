# Money — Business Requirements Document (BRD)

| Field           | Value                                                                |
| --------------- | -------------------------------------------------------------------- |
| Status          | Draft                                                                |
| Version         | 0.1.0                                                                |
| Scope           | `libs/money` · Phase 0                                               |
| Source of truth | [TS001 — Money Specification](./TS001_money-type.md) (authoritative) |

> This BRD captures the **business** need behind the Money library — the *what*
> and the *why*. The technical design (data model, parsing algorithm, public API,
> error model) and the acceptance criteria live in
> [TS001 — Money Specification](./TS001_money-type.md), which governs if the two ever disagree.

---

## 1. Purpose

Define a single, exact, currency-aware money value type that the application can
use to **store**, **parse**, and **display** retailer prices reliably. The goal
is to eliminate precision loss and currency confusion wherever monetary amounts
flow through the system.

## 2. Background & business context

Retailer offers are scraped from sites worldwide and priced in the retailer's
local currency. Those prices arrive as free-form text in many national number
and currency formats, and they must be stored and later shown back to users.
Today there is no shared, exact representation of money, which risks rounding
errors, mis-parsed amounts, and one currency being mistaken for another.
Converting between currencies (foreign-exchange) is a separate, system-level
concern handled elsewhere (see [phase0-roadmap.md](./phase0-roadmap.md)) and is
**not** part of this work.

## 3. Business objectives

- **O-1 — Exactness.** Represent and compute monetary amounts with no
  floating-point rounding error.
- **O-2 — Faithful ingestion.** Parse scraped prices in every number/currency
  format used across the supported currencies' countries.
- **O-3 — Currency safety.** Make it impossible to silently mix or mis-attribute
  currencies.
- **O-4 — Trustworthy display.** Render amounts back in a locale-appropriate,
  human-readable form.
- **O-5 — Predictability.** Fail loudly and specifically on bad input rather than
  guessing.

## 4. Delivery phases

The work is sequenced into three phases. Each business requirement and risk
below is tagged with the phase that delivers it; the requirements themselves are
not dropped, only sequenced.

### Phase 1 — MVP: ingest & store positive prices exactly

Deliver the scrape → parse → store pipeline end-to-end:

- The value type, exact integer storage, explicit currency, and the core
  non-functional guarantees. (BR-1, BR-2, NFR-1–4)
- Construction, accessors, and structural equality + hashing for storage and
  de-duplication — **no arithmetic**.
- Parsing of **positive** amounts for each currency, in both input modes (with and
  without an embedded currency indicator), covering every number-format family
  (A Anglo / B Central-EU / C Spaced-EU) at least once per currency (~7 locales:
  en-US/en-IE/en-CA/en-AU, de-DE, fr-FR/fr-CA), with currency validation against
  the expected currency. The full `ParseError` enumeration is defined here, even
  for variants only reachable once negatives arrive. (BR-3 baseline, BR-4, BR-6)
- Canonical serialization/deserialization for persistence.

### Phase 2 — Confident post-MVP

Complete the confidently-specified feature set:

- Negative amounts (leading/trailing minus, accounting parentheses, `-€`, U+2212)
  and the remaining locales/edge cases — completing BR-3 and delivering BR-5.
- Locale-appropriate display across the full locale matrix, with the
  format↔parse round-trip guarantee. (BR-7)
- The full arithmetic suite (add/subtract/negate/abs, scaling, allocation,
  min/max, comparison) and rounding modes for noisy fractional input. (BR-8, BR-9)

### Phase 3 — Discussion / decision backlog (not committed)

Open product/architecture questions to settle before they can be scoped — see
§13 Risks & open questions. None are committed for delivery: currency
auto-detection, non-2-decimal currency expansion, formatting ownership (library
vs presentation layer), messy-input policy, serialized-wire compatibility, and
anything FX / tax-VAT / cross-system multi-currency.

## 5. Stakeholders & consuming systems

Derived from the consumers the specification references. These are the parties
who depend on the Money type:

| Stakeholder / system            | Interest in Money                                                                          |
| ------------------------------- | ------------------------------------------------------------------------------------------ |
| Scraping / ingestion pipeline   | Produces the free-form price text that Money must parse.                                   |
| Pricing / offer layer           | Stores offer prices; may additionally enforce non-negativity on top of Money.              |
| Presentation / web & API layer  | Renders amounts for users; consumes formatting (or its own, if display later moves there). |
| Tax / VAT and other calculators | Consume Money values; tax logic itself is out of scope for this type.                      |
| Engineering (library owners)    | Build and maintain `libs/money` to the specification.                                      |

## 6. Scope

Scope below is the **overall** deliverable; §4 sequences it across phases.

### 6.1 In scope

- A money value pairing an **amount** with a **currency**.
- Supported currencies: **USD, EUR, CAD, AUD** (all with 2-decimal minor units).
- Exact storage of amounts as whole minor units (cents) — never floating point.
- Parsing an amount from any number format used by the countries that use those
  currencies, including negative amounts.
- Basic exact arithmetic and comparison **within a single currency**.
- Rendering a money value back to a locale-appropriate string.

### 6.2 Out of scope

- Currency conversion / exchange rates (system-level concern — see roadmap).
- Any currency beyond the four listed.
- Currencies with non-decimal or non-2-digit minor units (e.g. JPY, KWD, BHD) —
  though the design must not preclude adding them later.
- Cryptocurrencies and historical currencies.
- Tax/VAT logic (a consumer of this type, not part of it).

## 7. Business requirements

Each requirement below is mandatory **for the phase it is tagged with** (see §4
Delivery phases) — the phasing, not the requirement, is what is sequenced. Each
ID (BR-n) matches the specification so the two stay traceable; the *Why* line
gives the business rationale and any objective (O-n) it serves.

#### BR-1 · Explicit currency on every value · Phase 1

Every money value carries an explicit currency; a value cannot exist without
one. *Why:* prevents currency-less amounts that could be misread.

#### BR-2 · Exact integer storage · Phase 1

Amounts are stored as a whole number of minor units (cents); no floating point
at any point. *Why:* guarantees exactness (O-1).

#### BR-3 · Robust parsing of scraped prices · Phase 1 baseline → Phase 2 full coverage

Given the expected currency, parse a price written in every supported
number/currency format, including negative amounts. *Why:* scraped data is
heterogeneous; ingestion must not drop valid prices (O-2).

#### BR-4 · Currency validation on input · Phase 1

Validate currency symbols/codes embedded in the text, and reject any that
contradict the expected currency. *Why:* currency safety on input (O-3).

#### BR-5 · Negative amounts · Phase 2

Recognise negative amounts (minus signs and accounting parentheses). *Why:*
refunds, discounts, and deltas appear in real scraped data.

#### BR-6 · Predictable failure · Phase 1

Fail with a specific, predictable reason rather than producing a wrong amount or
crashing. *Why:* predictability and safe ingestion (O-5).

#### BR-7 · Locale-appropriate display · Phase 2

Render a money value to a human-readable, locale-appropriate string. *Why:*
trustworthy display (O-4).

#### BR-8 · Single-currency math · Phase 2

Allow **monetary** arithmetic and amount comparison only within one currency;
mixing currencies is an error, and currencies are never implicitly converted.
Different-currency values are still distinct values: the type provides a
deterministic ordering so they can be stored in sorted collections or used as
keys, but that ordering is not a monetary comparison and implies no exchange
relationship. *Why:* currency safety (O-3); avoids meaningless cross-currency
math while keeping values usable in data structures.

#### BR-9 · No silent precision loss · Phase 2

Never silently lose precision; any operation that must round exposes its
rounding behaviour explicitly. *Why:* exactness and auditability (O-1).

## 8. Non-functional requirements

#### NFR-1 · Exact & deterministic · Phase 1

Identical input always yields identical output, with no floating-point rounding
error and no dependence on the host runtime's Unicode version (text
normalization is pinned to a built-in table).

#### NFR-2 · Safe · Phase 1

All arithmetic is overflow-checked; values never silently wrap around.

#### NFR-3 · Immutable · Phase 1

Operations return new values rather than mutating their inputs.

#### NFR-4 · Self-contained · Phase 1

Currency and formatting data are built in; no runtime locale database is
required.

## 9. Assumptions

- The four supported currencies all use a 2-decimal minor unit (cents).
- At parse time the caller knows and supplies the expected currency (necessary
  because a bare `$` is shared by USD, CAD, and AUD).
- Free-form scraped text is the primary input the parser must handle.
- Foreign-exchange conversion and cross-system display strategy are owned
  elsewhere (the roadmap), not by this library.

## 10. Constraints

- Amounts fit a signed 64-bit minor-unit range — far beyond any realistic price
  (the specification notes roughly ±$9.2×10¹⁶) — and operations are
  overflow-checked rather than wrapping.
- Only 2-decimal currencies are supported today, but the design must not preclude
  later adding currencies with other minor-unit sizes (e.g. JPY, BHD).
- Locale and currency data are compiled-in constants; no runtime locale database.
- No foreign-exchange conversion is performed by this library.

## 11. Dependencies

- [phase0-roadmap.md](./phase0-roadmap.md) — owns the system-wide multi-currency
  question (FX conversion and display strategy) that this type deliberately
  excludes.
- The consuming systems in §5 rely on this type's behaviour.

## 12. Success criteria

The library meets the business need when, for the four supported currencies:

- Prices are stored and round-tripped with **zero precision loss**.
- Every supported real-world price format parses to the correct amount, and
  malformed or currency-contradicting input is rejected with a specific reason.
- Amounts render back into the correct locale-appropriate form.
- No operation can silently mix currencies or overflow.

These outcomes are verified by the acceptance criteria in §3 of
[TS001 — Money Specification](./TS001_money-type.md), which serve as the
executable definition of done.

## 13. Risks & open questions

- **Currency auto-detection (open · Phase 3).** Callers must currently supply the
  expected currency because a bare `$` is ambiguous across USD/CAD/AUD. Whether
  the system will need to infer currency from the text alone is unresolved
  (Decision status item 5 in the spec). All other earlier design decisions are
  settled.
- **Messy scraped input (Phase 3 policy).** Real-world price text is noisy; the
  reject-by-default-with-specific-errors stance (BR-6) mitigates silent corruption
  but means some inputs are rejected rather than guessed.
- **Future currency expansion (Phase 3).** Adding currencies with different
  minor-unit sizes is anticipated; the design keeps the door open, but expansion
  is a future change, not a current commitment.

## 14. Glossary

- **Minor units (cents):** the smallest unit of a currency; amounts are stored as
  an exact whole count of them.
- **Locale:** the regional convention that determines how an amount is written and
  displayed (separators, symbol position).
- **Foreign exchange (FX):** converting an amount from one currency to another —
  out of scope here.
- **Round-trip:** formatting an amount to a string and parsing it back yields the
  original value.

## 15. References

- [TS001 — Money Specification](./TS001_money-type.md) — the authoritative specification (normative
  requirements, technical design, error model, acceptance criteria).
- [phase0-roadmap.md](./phase0-roadmap.md) — system-wide multi-currency / FX
  roadmap item.
