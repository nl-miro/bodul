# Breaking changes / spec deviations (Claude, Phase 1)

This file records where the Phase 1 implementation deviates from, or could not
literally satisfy, the specs (TS001 / IS001 / BR001) — per the request to document
anything that "doesn't fit" rather than silently change the contract.

## Summary

**No public-API change was required.** The skeleton's types and signatures
(`Currency`, `Money`, `ParseOptions`, `RoundingMode`, the error enums, and every
function signature) matched TS001 §2.9 / IS001 §3 exactly and were implemented as-is.

The only deviation is a single acceptance-criterion row that is internally
inconsistent with the normative parsing algorithm, documented below.

## Deviation 1 — AC-P-NEG-9 (`12.999` → `TooManyFractionalDigits`) not honoured

**What the spec says (conflicting):**

- **AC-P-NEG-9** (TS001 §3): expected `USD` `12.999` (">2 dp, reject mode") →
  `TooManyFractionalDigits`.
- **AC-P-AMB-5** (TS001 §3): expected `USD` `12.345` → `1234500` (grouping).
- **§2.4 step 6**: for a single separator appearing once with exactly **3** trailing
  digits (`k == 1 && d == 3`), the separator is the **group** separator and the value
  has **no fractional part** ("the ambiguity default").
- **Decision #3** (TS001 §4, status `decided`): the single-separator 3-trailing-digit
  case (`1.234` / `1,234`) is *treated as grouping*; the only other reading (a 3-dp
  decimal) is invalid for 2-dp currencies, "so the risk is nil".

`12.999` is structurally identical to `12.345`: one `.`, appearing once, with three
trailing digits. The normative §2.4 algorithm and Decision #3 require it to be read as
**grouping** → `12999` integer units → `1299900` minor units. There is no
deterministic rule by which `12.345` is grouping while `12.999` is a rejected
fraction; the two differ only in digit values, which the algorithm never inspects.

**What this implementation does:**

`Money::parse("12.999", USD, default)` returns `Ok(Money(1299900, USD))`, consistent
with §2.4 step 6, Decision #3, and AC-P-AMB-5. AC-P-NEG-9 as written is **not**
reproduced.

**Why TS001 governs:** IS001 §1 and the TS001 header both state TS001 governs on any
conflict, and the §2.4 algorithm + Decision #3 are the normative procedure. AC-P-NEG-9
appears to be an erroneous example (a genuinely-fractional `TooManyFractionalDigits`
input must have its decimal separator disambiguated as a decimal — e.g. both
separators present, or 1–2 trailing digits). The test suite instead exercises the
`TooManyFractionalDigits` path with `1.234,567` (rightmost `,` is unambiguously the
decimal separator, leaving a 3-digit fraction) and asserts `12.999 → 1299900`.

**Note for Phase 2:** AC-P-RND-1…4 treat `12.999` / `12.994` / `12.995` / `12.985` as
fractions to round when a `RoundingMode` is supplied. That is a separate
rounding-enabled code path (Phase 2) and is compatible with the reject-mode grouping
default above only if rounding mode changes how 3-trailing-digit inputs are
interpreted. Phase 2 should resolve this explicitly (and ideally correct or remove
AC-P-NEG-9 in TS001).

## Update — sign extraction added (§2.4 step 2), negatives gated

Initially this implementation deferred §2.4 step-2 sign extraction entirely, so
sign-bearing inputs fell through to the step-5 whitelist as `InvalidCharacter`. To
align with the shared cross-implementation acceptance suite (and the sibling PRs),
step-2 sign extraction is now implemented: leading/trailing `-`, leading `+`,
accounting parentheses, and U+2212 are detected and stripped (conflicting/duplicate
markers → `MalformedSign`). A **successful negative value is gated** — it returns
`MalformedSign` rather than a signed `Money`, because returning negative amounts is
Phase 2. Net effect on error classification:

- `-$` → `MalformedNumber` (sign + indicator stripped, no digit remains) — was
  `InvalidCharacter`.
- `-$5.00`, `($5.00)`, `−5,00 €` → `MalformedSign` (negative gated) — was
  `InvalidCharacter`.

The full-width fold (digits, `＄`/`．`/`，`, plus U+2212 → `-`) is also implemented, so
AC-P-26 (`＄１２．３０` → 1230) parses.

## Non-deviations worth noting (resolved within Phase 1, no change needed)

- **AC-P-NEG-14** (`1.234,56 € EUR` → `MalformedCurrency`): §2.4 step 3 consumes at
  most one indicator per side, so the second `€` would otherwise survive as a stray
  character. Implemented faithfully by detecting a *leftover* indicator after
  extraction and returning `MalformedCurrency` (a bare number never matches an
  indicator, so valid amounts are unaffected).
- **AC-P-31** (`$5.00 CAD` → `500`) is a Phase 2 row (not in the IS001 Phase 1
  baseline). Phase 1's simpler "two distinct indicator tokens → `MalformedCurrency`"
  rule would reject it; refining bare-`$` + compatible-ISO acceptance is Phase 2 work.
