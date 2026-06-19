# Breaking Changes and Spec Deviations - Codex

## Summary

No public API changes were required. The existing Phase 1 skeleton signatures
still fit the implementation.

Two TS001 acceptance rows conflict with the normative parser rules. The
implementation follows the parser rules consistently and records those conflicts
here rather than hard-coding individual literals.

## AC-P-NEG-9: `12.999`

TS001 AC-P-NEG-9 expects `12.999` to return `TooManyFractionalDigits`, but
TS001 §2.4 step 6 and Decision #3 say a single separator with exactly three
trailing digits is treated as grouping. That makes `12.999` structurally
equivalent to AC-P-AMB-5 `12.345`, which parses as grouping.

Chosen behavior:

- `12.999` parses as `Money(1299900, USD)`.
- Neighboring inputs such as `12.998` and `13.999` follow the same grouping rule.
- No digit-value-specific special case is applied.

## AC-P-31: `$5.00 CAD`

TS001 AC-P-31 expects `$5.00 CAD` to parse as CAD 500, but TS001 §2.4 step 3
says two distinct consumed indicator tokens produce `MalformedCurrency`.
Applying a CAD-only exception would make `$5.00 CAD` succeed while identical
USD/AUD patterns fail.

Chosen behavior:

- `$5.00 CAD`, `$5.00 USD`, and `$5.00 AUD` all return `MalformedCurrency`.
- Single-indicator forms such as `$5.00`, `CAD$5.00`, and `US$5.00` remain
  supported according to the expected currency.
