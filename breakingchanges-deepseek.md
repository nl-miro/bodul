# Breaking Changes — Phase 1 Skeleton → Implementation

This file documents any changes to the Phase 1 skeleton's public surface that were
required during the Phase 1 implementation. The skeleton was designed to match
TS001/IS001 exactly; changes below are limited to internal additions and
corrections.

## Summary

**No breaking changes to the public API.** The skeleton's public contract
(types, function signatures, error variants, and visibility) was correct as
scaffolded. Implementation filled in `todo!()` bodies without altering any
public signature.

## Changes Made

### 1. `Currency` — added `PartialOrd` + `Ord` derives (non-breaking)

The skeleton derived `Debug, Clone, Copy, PartialEq, Eq, Hash`. `PartialOrd`
and `Ord` were added to enable total ordering of `(currency, amount_minor)` per
TS001 §2.6. This is a purely additive derive — existing consumers are unaffected.

### 2. `Currency` — added internal helpers (not public, no API change)

- `pub(crate) fn iso_code(self) -> &'static str` — ISO 4217 alpha code.
- `pub(crate) fn from_iso_code(code: &str) -> Option<Currency>` — reverse lookup.
- `pub(crate) const ALL_INDICATORS: &[(&str, Currency)]` — parser symbol table.

These are `pub(crate)` only and do not affect the public API surface.

### 3. `Currency` variant order changed (non-breaking, intentional)

Variants were reordered from `{USD, EUR, CAD, AUD}` to `{AUD, CAD, EUR, USD}`
so `#[derive(PartialOrd, Ord)]` produces the ISO 4217 alpha-code ordering
(`AUD < CAD < EUR < USD`) required by TS001 §2.6 for collection ordering.

**Impact:** If any consumer matches on `Currency` discriminant values rather
than named variants, the discriminant values changed. Named-variant matching is
unaffected. No known consumers existed at the time of this change.

### 4. Error `Display` impls — messages added (non-breaking)

The skeleton used `write!(f, "{self:?}")` as a placeholder. These were replaced
with human-readable messages per variant. The `Display` trait contract is
fulfilled either way, so this is not a breaking change.

## Deliberately Not Changed

- **`Money::parse` parameter `options: ParseOptions`** is required (not
  `Option<ParseOptions>`). TS001 pseudocode shows `options?` but the Rust
  skeleton uses `Default`-derived `ParseOptions` instead. Callers pass
  `ParseOptions::default()` for the no-options case. This is idiomatic Rust and
  was part of the IS001-specified skeleton — no change needed.

- **`serialize`/`deserialize` are inherent methods** on `Money`, not free
  functions. IS001 §3 specifies this Rust-specific adaptation of the TS001
  pseudocode. No change needed.

- **No `Display` impl on `Money`** (per TS001 implementation notes — human
  output goes through Phase 2 `format()`). No change needed.

- **No `serde` dependency.** Hand-rolled serialization per IS001. No change
  needed.

## Verification

- `cargo build` succeeds.
- `cargo doc --no-deps` renders the public API surface.
- All TS001 Phase 1 acceptance criteria (AC-P baseline positives, AC-P-AMB,
  AC-P-NEG, AC-P-ZERO-3/4, AC-S, AC-A-18, AC-A-19) are implemented.
