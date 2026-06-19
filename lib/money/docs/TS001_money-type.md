# Money — Specification

Status: Draft · Version: 0.1.0 · Scope: `libs/money` · phased delivery

Defines how monetary amounts are represented, stored, parsed, and rendered in
the application. Covers the currencies **USD, EUR, CAD, AUD**.
Language-agnostic: data structures and algorithms are described abstractly; see
[Implementation notes](#implementation-notes-non-normative) for the mapping to
the Rust crate.

This complements the multi-currency open question in
[phase0-roadmap.md](./phase0-roadmap.md) — that question covers FX conversion
and display strategy across the whole system; this document covers the core
value type. **FX conversion is explicitly out of scope here.**

For the business context, objectives, and phased delivery plan, see [BR001 — Business Requirements](./BR001_initial-business-requirements.md).

---

## 1. Business requirements

### 1.1 Context

Retailer offers are scraped from sites worldwide and priced in the retailer's
local currency. We need a single, exact, currency-aware money type to store
those prices, parse them out of free-form scraped text, and render them back for
display.

### 1.2 In scope

Scope below is the overall `libs/money` deliverable.
[Delivery phases](#14-delivery-phases) define when each capability is expected.

- A money value that pairs an **amount** with a **currency**.
- Currencies: **USD, EUR, CAD, AUD** (all minor-unit exponent 2).
- Exact storage as integer **minor units (cents)** — never floating point.
- **Parsing** an amount from any number format used by the countries that use
  those currencies (see §2.3).
- Basic exact arithmetic and comparison within a single currency.
- Rendering a money value back to a locale-appropriate string.

### 1.3 Out of scope

- Currency conversion / exchange rates (system-level concern — see roadmap).
- Currencies other than the four listed.
- Currencies with non-decimal or non-2-digit minor units (e.g. JPY, KWD,
  BHD). The design should not preclude adding them later.
- Cryptocurrencies, historical currencies.
- Tax/VAT logic (a consumer of this type, not part of it).

### 1.4 Delivery phases

The business requirements document sequences delivery in three phases. This spec
defines the complete target behavior, while this section identifies the minimum
surface required in each phase.

#### Phase 1 — MVP: ingest and store positive prices exactly

Phase 1 delivers the scrape → parse → store path:

- `Money(amount_minor, currency)` for USD, EUR, CAD, and AUD.
- Signed integer minor-unit storage, explicit currency, construction, accessors,
  structural equality, and hashing.
- Basic parsing of **positive** amounts when the caller supplies the expected
  currency. For each supported currency, Phase 1 includes at least one parse case
  with no embedded currency indicator and one parse case with an embedded
  currency indicator, and covers **every number-format family (A Anglo, B
  Central-EU, C Spaced-EU) at least once per currency** (the ~7 locales
  en-US/en-IE/en-CA/en-AU, de-DE, fr-FR/fr-CA).
- Currency-indicator validation and typed parse failures. The **full `ParseError`
  enumeration (§2.7) is defined in Phase 1**, including the sign/parentheses
  variants that only become reachable once Phase 2 adds negative amounts.
- Canonical serialization/deserialization for persistence.
- Phase 1 does **not** deliver monetary arithmetic, negative amount parsing,
  locale-aware formatting, or rounding options.

#### Phase 2 — Confident post-MVP

Phase 2 completes the confidently specified library behavior:

- Negative amount parsing and remaining parse edge cases.
- Locale-aware formatting and format↔parse round-trip behavior.
- Exact arithmetic, scaling, allocation, min/max, semantic comparison, total
  ordering, and explicit rounding modes.
- Full non-functional hardening around overflow, deterministic normalization,
  and edge-case acceptance criteria.

#### Phase 3 — Discussion / decision backlog

Phase 3 contains topics that are not committed for delivery until product or
architecture decisions are made:

- Currency auto-detection without caller-supplied expected currency.
- Expansion beyond USD/EUR/CAD/AUD, especially zero- or three-exponent
  currencies.
- Moving user-facing formatting out of `libs/money` into the presentation layer.
- Compatibility policy if serialized money has earlier JSON-number producers.
- Messy scraped-input policy beyond the current reject-by-default stance.
- FX conversion, exchange-rate display, tax/VAT logic, and broader cross-system
  multi-currency strategy.

### 1.5 Functional requirements

Each requirement is mandatory for the phase listed below.

| ID   | Phase                      | Requirement                                                                                                                                                                                                                                                                                                                                                                    |
| ---- | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| BR-1 | Phase 1                    | A money value is the pair `(amount_minor, currency)`; a value cannot exist without an explicit currency.                                                                                                                                                                                                                                                                       |
| BR-2 | Phase 1                    | Amounts are stored as a signed integer count of minor units (cents). No floating-point storage at any point.                                                                                                                                                                                                                                                                   |
| BR-3 | Phase 1 baseline → Phase 2 | The system parses an amount string into a money value given the expected currency. Phase 1 covers positive baseline parsing for each supported currency, with and without embedded currency indicators; Phase 2 completes every number format listed in §2.3, including negative amounts and edge cases.                                                                       |
| BR-4 | Phase 1                    | Parsing recognises and validates currency symbols / ISO codes embedded in the input and rejects inputs whose currency contradicts the expected currency.                                                                                                                                                                                                                       |
| BR-5 | Phase 2                    | Parsing recognises negative amounts (leading/trailing minus and accounting parentheses).                                                                                                                                                                                                                                                                                       |
| BR-6 | Phase 1                    | Parsing fails with a specific, typed error rather than producing a wrong amount or panicking.                                                                                                                                                                                                                                                                                  |
| BR-7 | Phase 2                    | The system renders a money value to a human-readable, locale-appropriate string.                                                                                                                                                                                                                                                                                               |
| BR-8 | Phase 2                    | **Semantic** arithmetic and amount comparison (`add`, `sub`, `min`, `max`, `try_cmp`) are supported only between values of the **same** currency; mixing currencies returns `CurrencyMismatch`. **Structural** equality, hashing, and total ordering are total over the whole `(amount_minor, currency)` value (for collection/key use) and do not imply exchange equivalence. |
| BR-9 | Phase 2                    | No operation silently loses precision; any operation that must round exposes the rounding behaviour explicitly.                                                                                                                                                                                                                                                                |

### 1.6 Non-functional requirements

- **Exactness & determinism (Phase 1):** identical input always yields identical output;
  no rounding error from binary floating point; no dependence on the host
  runtime's ambient Unicode tables (see §2.4 step 0).
- **Safety (Phase 1 baseline, Phase 2 arithmetic):** all arithmetic is
  overflow-checked; no silent wraparound.
- **Immutability (Phase 1):** a money value is an immutable value type;
  operations return new values.
- **Locale data is static (Phase 1):** the currency/format tables are
  compiled-in constants, no runtime locale database dependency.

---

## 2. Technical specification

### 2.1 Data model

```
Currency  = enum { USD, EUR, CAD, AUD }

Money     = struct {
    amount_minor : SignedInteger(>= 64 bits)   // count of minor units (cents)
    currency     : Currency
}

Locale    = enum {
    EN_US, EN_IE, EN_CA, EN_AU,
    DE_DE, IT_IT, ES_ES, NL_NL, DE_AT,
    FR_FR, FI_FI, PT_PT, FR_CA
}

ParseOptions = struct {
    rounding : Optional<RoundingMode> = None
}
```

- `amount_minor` is a **signed** 64-bit integer. Signed so the type can also
  represent deltas, refunds, and discounts; price-level callers MAY additionally
  enforce non-negativity (see [Decision status](#4-decision-status)).
- A 64-bit integer spans roughly ±9.2×10¹⁸ minor units (±$9.2×10¹⁶), far beyond
  any realistic price.
- `Money` carries no locale; locale is supplied per parse/format call.
- `Money` fields are private. Values are created through the constructors in
  [Public interface](#29-public-interface-pseudocode-non-normative-shape) and
  inspected through accessors, so callers cannot bypass invariants by raw struct
  construction.
- `Locale` is a closed enum of supported formatting locales. If a language API
  accepts external BCP-47 locale tags, it MUST validate them against this static
  allowlist and return `UnsupportedLocale` for unknown tags.
- When `format` omits `locale`, it uses the currency default from
  [Formatting](#28-formatting-rendering).
- `Currency::exponent()` is the source of truth for fractional digit count. It
  returns `2` for every currently supported currency.

### 2.2 Currency definitions

All four supported currencies have a minor-unit exponent of **2** (100 minor
units = 1 major unit), so "cents" is uniform across them.

| Currency          | ISO code | ISO num | Exponent | Symbols accepted on input               |
| ----------------- | -------- | ------- | -------- | --------------------------------------- |
| US Dollar         | USD      | 840     | 2        | `$`, `US$`, `USD`                       |
| Euro              | EUR      | 978     | 2        | `€`, `EUR`                              |
| Canadian Dollar   | CAD      | 124     | 2        | `$`, `C$`, `CA$`, `Can$`, `CAD$`, `CAD` |
| Australian Dollar | AUD      | 036     | 2        | `$`, `A$`, `AU$`, `AUD$`, `AUD`         |

> `$` is **ambiguous** — it is valid for USD, CAD, and AUD. A bare `$` therefore
> never *determines* the currency; the caller-supplied expected currency is
> authoritative (see the currency-indicator extraction rule in §2.4). `US$`,
> `C$`, `CA$`, `Can$`, `CAD$`, `A$`, `AU$`, `AUD$`, `€`, and all ISO codes are
> unambiguous. `US$` is primarily a disambiguation aid; most US retailer input
> uses bare `$`.
> Reverse spellings such as `$CA` and `$AU` are not accepted unless explicitly
> added to this table.

### 2.3 Number-format matrix (the inputs we must parse)

Across the countries using these currencies, three distinct numeric conventions
appear. Every other listed locale is one of these three families.

| Family             | Group separator | Decimal separator | Symbol position       | Example                     |
| ------------------ | --------------- | ----------------- | --------------------- | --------------------------- |
| **A** — Anglo      | `,` (comma)     | `.` (dot)         | leading               | `$1,234.56` / `€1,234.56`   |
| **B** — Central-EU | `.` (dot)       | `,` (comma)       | trailing (or leading) | `1.234,56 €` / `€ 1.234,56` |
| **C** — Spaced-EU  | space*          | `,` (comma)       | trailing              | `1 234,56 €` / `1 234,56 $` |

`*` "space" means any of: ASCII space `U+0020`, no-break space `U+00A0`, narrow
no-break space `U+202F`, thin space `U+2009`, figure space `U+2007`.

Currency → families seen in the wild (informative — the parser auto-detects and
need not be told which):

| Currency | Families | Representative locales                                                                                                       |
| -------- | -------- | ---------------------------------------------------------------------------------------------------------------------------- |
| USD      | A        | United States (en-US)                                                                                                        |
| EUR      | A, B, C  | Ireland (en-IE) → A; Germany, Italy, Spain, Netherlands, Austria (de/it/es/nl) → B; France, Finland, Portugal (fr/fi/pt) → C |
| CAD      | A, C     | English Canada (en-CA) → A; French Canada (fr-CA) → C                                                                        |
| AUD      | A        | Australia (en-AU) → A                                                                                                        |

#### 2.3.1 Locale → family mapping

Every `Locale` variant maps to exactly one formatting family. This table is the
single source of truth the `format` function (§2.8) uses to select group
separator, decimal separator, and symbol position.

| Locale | Family |
| ------ | ------ |
| EN_US  | A      |
| EN_IE  | A      |
| EN_CA  | A      |
| EN_AU  | A      |
| DE_DE  | B      |
| IT_IT  | B      |
| ES_ES  | B      |
| NL_NL  | B      |
| DE_AT  | B      |
| FR_FR  | C      |
| FI_FI  | C      |
| PT_PT  | C      |
| FR_CA  | C      |

Negative-amount notations to accept (any family):

- Leading minus: `-1.234,56 €`
- Trailing minus: `1.234,56-`
- Accounting parentheses: `(1,234.56)` / `($5.00)`

Out of scope for parsing: apostrophe grouping (`1'234.56`, Swiss CHF — not a
supported currency) and Indian grouping (`12,34,567`). These MUST be rejected,
not mis-parsed.

### 2.4 Parsing algorithm

Input: `raw` (string), `currency` (the expected `Currency`), `options`
(`ParseOptions`; default = reject extra fractional digits — see §2.5). Output:
`Money` or a typed `ParseError` (§2.7).

0. **Pre-normalization and length guard.** If `raw` is longer than 256 Unicode
   scalar values — measured on the **raw input before normalization**, as a cheap
   denial-of-service bound — return `InputTooLong`. Then apply this **explicit,
   version-stable** fold map (the parser MUST NOT depend on the host runtime's
   ambient Unicode/NFKC tables, so results are identical across runtimes and
   Unicode versions):

| From                                         | To                            |
| -------------------------------------------- | ----------------------------- |
| U+FF10–U+FF19 (full-width digits)            | U+0030–U+0039 (ASCII `0`–`9`) |
| U+FF04 `＄` (full-width dollar)              | U+0024 `$`                    |
| U+FF0E `．` (full-width full stop)           | U+002E `.`                    |
| U+FF0C `，` (full-width comma)               | U+002C `,`                    |
| U+2212 (minus sign)                          | U+002D `-`                    |
| U+00A0, U+2009, U+202F, U+2007 (§2.3 spaces) | U+0020 (ASCII space)          |

Any character not in this map is left unchanged; unlisted full-width forms and
any other whitespace therefore survive to fail the step-5 whitelist as
`InvalidCharacter`. §2.3's five-space list enumerates the spaces that may appear
in **raw input**; after this fold the only space character that can remain is
ASCII U+0020 (which step 4 then validates and removes).
1. **Trim** leading/trailing whitespace. If empty → `EmptyInput`.
2. **Sign extraction.** Record negative if the value is wrapped in outer
   accounting parentheses, has a leading `-`, has a trailing `-` (the last
   non-space character), or has a `-` immediately preceding a trailing currency
   indicator with zero or more spaces between them. A leading `+` is positive.
   Strip the sign/paren markers and re-trim surrounding whitespace. At most one
   sign indicator may be present → else `MalformedSign`. Legal examples include
   `-$5.00`, `($5.00)`, `(€5,00)`, `-1.234,56 €`, `1.234,56 €-`, `5,00-€`,
   `1.234,56 -` (space before trailing minus), and `1.234,56 - €` (space before a
   minus that precedes a trailing indicator).
3. **Currency-indicator extraction (longest match).** Tokenize at most one
   leading and one trailing currency indicator by **longest match** over the
   configured symbol/ISO-code table (case-insensitive, ignoring spaces between
   indicator and number). Each side is consumed as a single token, so `CAD$` is
   the one token `CAD$`, never `CAD` + `$`.
   - If an **unambiguous** indicator (`€`, any ISO code, or `US$`/`C$`/`CA$`/
     `Can$`/`CAD$`/`A$`/`AU$`/`AUD$`) is present and it does not match
     `currency` → `CurrencyMismatch`.
   - A bare `$` is accepted iff `currency ∈ {USD, CAD, AUD}`; for EUR a bare `$`
     → `CurrencyMismatch`.
   - Strip the indicator(s). If two distinct indicator tokens are consumed (e.g.
     a leading and a trailing one that differ, as in `CAD$5.00 CAD`) →
     `MalformedCurrency`.
4. **Validate and remove group spacing.** Listed spaces act **only** as group
   separators. If any remain in the numeric payload, the space-delimited digit
   groups MUST satisfy the grouping rule — the first group is 1–3 digits and
   every later group exactly 3 digits (any decimal separator and fraction stay
   attached to the final group, so `1 234,56` has groups `1`/`234`). A violation
   such as `12 34,56` or `1 23 456` → `InvalidGrouping`. Then delete the spaces.
5. **Character whitelist and digit presence.** The remainder must contain only
   ASCII digits, `.`, and `,`; any other character → `InvalidCharacter`. It MUST
   also contain at least one ASCII digit — an input that is only signs, currency
   indicators, separators, or spaces (e.g. `$`, `USD`, `-$`) → `MalformedNumber`.
6. **Decimal-separator detection** (the core disambiguation):
   - **Both** `.` and `,` present → the **right-most** separator is the decimal
     separator. Exactly one decimal separator may be present, and every separator
     before it must be the same single group separator character. Mixed remaining
     separators such as `1,234.567,89` → `InvalidGrouping`.
   - **Only one** separator kind present (say `s`), appearing `k` times with
     `d` digits after its last occurrence:
     - `k > 1` → `s` is the **group** separator; there is no fractional part.
     - `k == 1` and `d == 3` → treat `s` as a **group** separator; a lone
       separator before exactly three trailing digits is the ambiguity default,
       and the value has no fractional part.
       *(Ambiguity default — overridable; see [Decision status](#4-decision-status).)*
     - `k == 1` and `d ∈ {1,2}` → `s` is the **decimal** separator.
     - `k == 1` and `d == 0` (e.g. `1,`) → `MalformedNumber`.
     - `k == 1` and `d >= 4` → `InvalidGrouping`.
   - **Neither** present → integer amount, no fractional part.
7. **Group and integer validation.** Split the integer part on the `.`/`,` group
   separator. With such a group separator present, every group after the first
   MUST be exactly 3 digits and the first 1–3 digits. With or without a group
   separator, a leading zero is allowed only when the integer part is exactly
   `0`. Violation → `InvalidGrouping`. Then remove group separators.
8. **Fraction handling.** Let `e = currency.exponent()` and `f` be the
   fractional digits (possibly empty):
   - `len(f) > e`:
     - default (reject) → `TooManyFractionalDigits`.
     - rounding mode set → round to `e` digits using the selected rule.
   - Right-pad `f` to exactly `e` digits (for the current exponent-2 currencies,
     `"5"`→`"50"` and `""`→`"00"`).
9. **Assemble minor units.** `digits = integer_digits + padded_fraction`;
   parse as integer; apply sign. If it exceeds the signed 64-bit range →
   `Overflow`. Zero is canonical: `-0.00`, `(0.00)`, and `+0.00` all yield
   `Money(0, currency)`.
10. Return `Money(amount_minor, currency)`.

> **Exponent note.** The fraction rule in step 8 is `currency.exponent()`-driven,
> so it already generalizes: a hypothetical exponent-0 currency (e.g. JPY) would
> reject any fractional digits, and an exponent-3 currency (e.g. BHD) would accept
> up to three. Only exponent-2 currencies are in scope today.

#### Worked disambiguation examples

| Input          | Decimal sep       | Reading                                     | Minor units       |
| -------------- | ----------------- | ------------------------------------------- | ----------------- |
| `1,234.56`     | `.` (right-most)  | group `,`                                   | `123456`          |
| `1.234,56`     | `,` (right-most)  | group `.`                                   | `123456`          |
| `1 234,56`     | `,`               | group space (validated, then removed)       | `123456`          |
| `1.234`        | none (3 trailing) | group `.`                                   | `123400`          |
| `1,234`        | none (3 trailing) | group `,`                                   | `123400`          |
| `1.23`         | `.` (2 trailing)  | decimal                                     | `123`             |
| `1,23`         | `,` (2 trailing)  | decimal                                     | `123`             |
| `12.345`       | none (3 trailing) | group `.`                                   | `1234500`         |
| `1.234.567,89` | `,` (right-most)  | group `.`                                   | `123456789`       |
| `1,2345`       | —                 | 4 trailing, not valid group, not ≤2 decimal | `InvalidGrouping` |
| `1.2.3`        | —                 | groups `1`/`2`/`3` malformed                | `InvalidGrouping` |
| `12 34,56`     | —                 | space group `12`/`34` (2nd ≠ 3 digits)      | `InvalidGrouping` |

### 2.5 Rounding & precision

- The **core stores and computes exact integer minor units**; no rounding occurs
  in storage, addition, subtraction, negation, or comparison.
- Rounding is only ever needed where a result has sub-minor-unit precision:
  parsing inputs with more than `currency.exponent()` fractional digits, and
  scalar multiplication by a non-integer/percentage.
- Supported rounding modes (caller-selected): `HALF_UP`, `HALF_EVEN` (banker's),
  `DOWN` (truncate), `UP`, `CEILING`, `FLOOR`.
- Rounding modes are defined by direction:
  - `DOWN` rounds toward zero.
  - `UP` rounds away from zero.
  - `FLOOR` rounds toward negative infinity.
  - `CEILING` rounds toward positive infinity.
  - `HALF_UP` rounds ties away from zero.
  - `HALF_EVEN` rounds ties to the nearest even minor unit.
- **Default for parsing = reject** (`TooManyFractionalDigits`) so scraped data is
  never silently corrupted. Callers that expect noisy data opt into a rounding
  mode explicitly.

### 2.6 Operations & invariants

| Operation        | Signature (pseudocode)                                                  | Rule                                                                                                       |
| ---------------- | ----------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| add              | `add(Money, Money) -> Result<Money, MoneyError>`                        | same currency else `CurrencyMismatch`; overflow-checked                                                    |
| subtract         | `sub(Money, Money) -> Result<Money, MoneyError>`                        | same currency; overflow-checked                                                                            |
| negate           | `neg(Money) -> Result<Money, MoneyError>`                               | overflow-checked; `neg(Money(0, currency)) == Money(0, currency)`                                          |
| absolute         | `abs(Money) -> Result<Money, MoneyError>`                               | `i64::MIN` -> `Overflow`; `abs(Money(0, currency)) == Money(0, currency)`                                  |
| semantic compare | `try_cmp(Money, Money) -> Result<Ordering, MoneyError>`                 | same currency required else `CurrencyMismatch`                                                             |
| total compare    | `cmp(Money, Money) -> Ordering`                                         | total ordering by `(currency, amount_minor)` for collection use; currencies ordered by ISO 4217 alpha code |
| scale by integer | `mul_int(Money, int) -> Result<Money, MoneyError>`                      | overflow-checked, exact; zero and negative multipliers are valid and carry the input currency              |
| scale by ratio   | `mul_ratio(Money, num, den, RoundingMode) -> Result<Money, MoneyError>` | rounds per mode (tax, discount); see ratio rules below                                                     |
| allocate         | `allocate(Money, n) -> Result<[Money], MoneyError>`                     | split into `n` parts whose **sum equals the original**; see allocation rules below                         |
| min / max        | `min/max(Money, Money) -> Result<Money, MoneyError>`                    | same currency required else `CurrencyMismatch`                                                             |

Invariants:

- **INV-1** Every `Money` has exactly one currency.
- **INV-2** Storage and exact arithmetic use integer minor units only — no float
  ever touches an amount.
- **INV-3** Cross-currency **semantic** arithmetic and amount comparison
  (`add`, `sub`, `min`, `max`, `try_cmp`) are impossible — rejected at the
  earliest possible point (compile time if the language allows, else a runtime
  `CurrencyMismatch`). **Structural** equality, hashing, and total ordering
  remain total over `(amount_minor, currency)` for collection use and do not
  imply exchange equivalence.
- **INV-4** No implicit currency conversion.
- **INV-5** `allocate` is lossless: `sum(allocate(m, n)) == m`.
- **INV-6** Round-trip: for any value rendered in family *X*, parsing the result
  back yields the original `Money` (see AC-F-3).

Standard equality and hashing are total over the full `(amount_minor, currency)`
tuple. Therefore `Money(500, USD) != Money(500, EUR)`, and `Money` is safe to
use as a hash key. Total ordering for sorted collections orders by `(currency,
amount_minor)`, where currencies are ordered by their ISO 4217 alpha code (so
the currency order is `AUD < CAD < EUR < USD`). This structural ordering is for
collection/key use only and does not imply any exchange-rate relationship.
Semantic amount comparison remains `try_cmp`, which returns `CurrencyMismatch`
for mixed currencies.

`mul_ratio(a, num, den, mode)` computes `round(a.amount_minor * num / den,
mode)`. `num` and `den` are **signed 64-bit integers**; a value outside that
range returns `Overflow`. **If `num == 0`, the result is `Money(0, currency)`
regardless of `den`, including `den == 0` — the zero numerator short-circuits
the division-by-zero check.** Otherwise `den == 0` returns `DivisionByZero`. The
product of two signed 64-bit integers always fits in a signed 128-bit integer,
so signed 128-bit arithmetic or an equivalent exact intermediate is sufficient
and recommended. When no wider integer type is available, the implementation
MUST detect intermediate overflow before division, for example with checked
multiply or a checked mul/div algorithm, and return `Overflow`; it MUST NOT
compute the product unchecked in the storage width. If the exact final result
cannot fit in signed 64-bit minor units, return `Overflow`. Negative numerators
and denominators are allowed; the sign follows standard signed-ratio arithmetic.

`allocate(a, parts)` has these preconditions and rules:

- `parts` MUST be a positive integer. `parts <= 0` returns `InvalidArgument`.
- `parts` MAY exceed `abs(a.amount_minor)`; zero-valued parts are valid.
- Let `base = a.amount_minor / parts` using truncation toward zero, and let
  `remainder = a.amount_minor - base * parts`.
- Allocate `base` to every part.
- For the first `abs(remainder)` parts, in index order, add one minor unit with
  the sign of `remainder`.
- This deterministic tie-break gives earlier indices the extra cent when
  remainders are equal, and the returned parts MUST sum exactly to the input.
- `amount_minor == i64::MIN` MUST be handled without panic and without forming
  `abs(i64::MIN)`. `abs(remainder)` is safe because `abs(remainder) < parts`.
  If an implementation cannot satisfy this safely, it MUST return `Overflow`.

### 2.7 Error model

Parsing, formatting, arithmetic, and (de)serialization expose typed errors.
`CurrencyMismatch` and `Overflow` may appear in both parse and arithmetic
contexts.

`ParseError`:

| Error                     | Meaning                                                                                                                                |
| ------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `EmptyInput`              | Input empty after trimming.                                                                                                            |
| `InputTooLong`            | Input exceeds the parser's maximum length before normalization.                                                                        |
| `InvalidCharacter`        | Disallowed character after separators handled.                                                                                         |
| `MalformedSign`           | Conflicting/duplicate sign markers.                                                                                                    |
| `MalformedCurrency`       | Multiple distinct currency indicator tokens.                                                                                           |
| `CurrencyMismatch`        | Embedded currency contradicts the expected currency.                                                                                   |
| `MalformedNumber`         | Separator present but no digits where required (e.g. `1,`), or no digit at all after sign/currency extraction (e.g. `$`, `USD`, `-$`). |
| `InvalidGrouping`         | Group sizes violate the 3-digit rule (including space groups), or unsupported grouping.                                                |
| `TooManyFractionalDigits` | More than `currency.exponent()` fractional digits with rounding disabled.                                                              |
| `Overflow`                | Amount (or arithmetic result) exceeds 64-bit range.                                                                                    |

`FormatError`:

| Error               | Meaning                                                 |
| ------------------- | ------------------------------------------------------- |
| `UnsupportedLocale` | Locale is not in the static supported-locale allowlist. |

`MoneyError`:

| Error              | Meaning                                                                           |
| ------------------ | --------------------------------------------------------------------------------- |
| `CurrencyMismatch` | Semantic arithmetic, comparison, min, or max received mixed currencies.           |
| `Overflow`         | Arithmetic result (or an out-of-range `mul_ratio` operand) exceeds 64-bit range.  |
| `DivisionByZero`   | `mul_ratio` was called with `den == 0` and `num != 0`.                            |
| `InvalidArgument`  | Argument violates an operation precondition, such as `parts <= 0` for `allocate`. |

`DeserializeError` (§2.10):

| Error                | Meaning                                                                                                                  |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `UnknownCurrency`    | `currency` is not a supported ISO 4217 alpha code.                                                                       |
| `InvalidAmountMinor` | `amount_minor` string is not a canonical base-10 signed integer (see §2.10).                                             |
| `AmountOutOfRange`   | `amount_minor` is a valid integer but outside the signed 64-bit range.                                                   |
| `MalformedWireValue` | Wire value is structurally wrong, e.g. `amount_minor` encoded as a JSON number rather than a string, or a missing field. |

### 2.8 Formatting (rendering)

`format(Money, locale?) -> Result<String, FormatError>` renders using the family
of the given locale (§2.3 / §2.3.1): correct group separator, decimal separator,
symbol, symbol position, and `currency.exponent()` fractional digits. The
exponent is `2` for all currently supported currencies. Default per-currency
locale when none is supplied: USD→en-US, EUR→de-DE, CAD→en-CA, AUD→en-AU.

Any supported locale may format any supported currency. Unsupported locales
return `UnsupportedLocale`; implementations MUST NOT silently fall back to a
different locale when the caller supplied one explicitly.

**Symbol selection rule.** The formatter always emits an **unambiguous**
indicator, chosen by whether the `(currency, family)` pair is **native** (some
real locale uses that currency in that family per §2.3):

- *Native, leading-symbol family:* the currency's leading symbol — `$` for USD
  (en-US), `€` for EUR (en-IE). CAD and AUD use their prefixed symbols `CA$` /
  `A$` in leading-symbol families to keep `$` unambiguous.
- *Native, trailing-symbol family:* the locale's conventional trailing form —
  `€` for EUR (Families B/C), and the bare trailing `$` for CAD in fr-CA.
- *Non-native pair:* the **ISO alpha code** (e.g. USD or CAD or AUD in Family B,
  USD or AUD in Family C).

AUD has no native Family-C locale (§2.3), so AUD in Family C is a non-native
pair and emits `AUD`; CAD's native fr-CA gives the bare trailing `$`. This is
why the CAD and AUD Family-C cells differ.

| Currency | Family A output | Family B output | Family C output |
| -------- | --------------- | --------------- | --------------- |
| USD      | `$1,234.56`     | `1.234,56 USD`  | `1 234,56 USD`  |
| EUR      | `€1,234.56`     | `1.234,56 €`    | `1 234,56 €`    |
| CAD      | `CA$1,234.56`   | `1.234,56 CAD`  | `1 234,56 $`    |
| AUD      | `A$1,234.56`    | `1.234,56 AUD`  | `1 234,56 AUD`  |

Negative formatted values use a leading minus before the number and symbol
cluster. Examples: `-$5.00`, `-1.234,56 €`, `-1 234,56 €`. **Zero always formats
without a minus sign, regardless of how the zero value was obtained** (e.g. via
`neg`), because zero is canonical (§2.4 step 9).

### 2.9 Public interface (pseudocode, non-normative shape)

The list below is the complete target interface. Phase 1 exposes the value type,
constructors/accessors, `parse`, canonical serialization/deserialization, and
structural equality/hash behavior. Phase 2 adds formatting, arithmetic,
allocation, semantic comparison, total ordering, and rounding-dependent
operations.

```
new(amount_minor: i64, currency: Currency) -> Money
from_major(units: i64, fractional_minor: i64, currency: Currency) -> Result<Money, MoneyError>
minor_units(m: Money) -> i64
currency(m: Money) -> Currency
currency_exponent(currency: Currency) -> u8

parse(raw: String, currency: Currency, options?: ParseOptions) -> Result<Money, ParseError>
format(value: Money, locale?: Locale) -> Result<String, FormatError>

serialize(m: Money) -> String                               // canonical JSON object, §2.10
deserialize(wire: String) -> Result<Money, DeserializeError> // exact, never the §2.4 parser

add(a: Money, b: Money)        -> Result<Money, MoneyError>
sub(a: Money, b: Money)        -> Result<Money, MoneyError>
neg(a: Money)                  -> Result<Money, MoneyError>
abs(a: Money)                  -> Result<Money, MoneyError>
mul_int(a: Money, n: Integer)  -> Result<Money, MoneyError>
mul_ratio(a: Money, num: i64, den: i64, r: RoundingMode) -> Result<Money, MoneyError>
allocate(a: Money, parts: Integer) -> Result<[Money], MoneyError>
try_cmp(a: Money, b: Money)    -> Result<Ordering, MoneyError>
cmp(a: Money, b: Money)        -> Ordering
```

`new` is infallible because every signed 64-bit minor-unit value plus a
supported currency is a valid `Money`. `from_major` combines whole major units
and a signed fractional minor-unit component using `currency.exponent()`.
`fractional_minor` MUST have magnitude less than `10^currency.exponent()`, and
`units` and `fractional_minor` MUST have the same sign unless either component
is zero; otherwise it returns `InvalidArgument`. This permits `from_major(0,
-34, USD) -> Money(-34, USD)` and makes `from_major(-12, -34, USD)` yield
`Money(-1234, USD)`. The conversion is overflow-checked.

`mul_ratio` takes signed 64-bit `num`/`den`; see §2.6 for the zero-numerator
short-circuit, `DivisionByZero`, and `Overflow` rules.

`serialize`/`deserialize` use the canonical wire format in §2.10 and MUST NOT
use the free-form parser in §2.4. A host language MAY instead provide these
through its serde-style integration, provided the same wire grammar and
`DeserializeError` behavior hold.

`ParseOptions` is a forward-extensible options struct. Its initial field is
`rounding: Optional<RoundingMode> = None`; `None` means reject inputs with too
many fractional digits.

### 2.10 Serialization

The canonical wire format is the raw data model:

```json
{ "amount_minor": "123456", "currency": "USD" }
```

- `amount_minor` is a base-10 **string**. Its grammar is exactly `-?[0-9]+` with:
  no leading `+`; no leading zeros except the single literal `0` (and `-0` is
  rejected — zero serializes as `"0"`); no surrounding or internal whitespace;
  non-empty. It is encoded as a string, not a JSON number, so values above the
  JSON safe-integer range round-trip exactly through JavaScript and other
  double-backed JSON parsers.
- `currency` is the ISO 4217 alpha code, not a symbol or numeric code.
- Serialization/deserialization MUST be exact and MUST NOT use the free-form
  parser in §2.4.
- Deserialization errors use `DeserializeError` (§2.7): an `amount_minor` encoded
  as a JSON *number* (rather than a string) is rejected as `MalformedWireValue`
  and MUST NOT be coerced; a string that violates the grammar (e.g. `"007"`,
  `"+5"`, `"12.5"`, `""`, `" 5 "`) → `InvalidAmountMinor`; a syntactically valid
  integer outside signed 64-bit range → `AmountOutOfRange`; an unknown `currency`
  code → `UnknownCurrency`.
- **Wire version.** This string encoding is the **v1** wire format; there is no
  earlier deployed consumer, so number-form `amount_minor` is rejected rather
  than accepted for backward compatibility. A future format change MUST be
  introduced as an explicit new version, not a silent reinterpretation.

---

## 3. Acceptance criteria

Format: each row is `(expected currency) input → expected result`. "→ X minor
units" means a successful parse to `Money(X, currency)`.

Acceptance criteria are phased:

- **Phase 1**: positive baseline parsing with expected currency supplied,
  currency validation, typed parse failures, canonical serialization, and NFRs.
- **Phase 2**: negative parsing, formatting and round-trip behavior, arithmetic,
  rounding, allocation, and total-order semantics.

### AC-P · Parsing — positive

| #       | Currency | Input                                        | Expected  |
| ------- | -------- | -------------------------------------------- | --------- |
| AC-P-1  | USD      | `$1,234.56`                                  | 123456    |
| AC-P-2  | USD      | `1234.56`                                    | 123456    |
| AC-P-3  | USD      | `US$12.30`                                   | 1230      |
| AC-P-4  | USD      | `USD 0.99`                                   | 99        |
| AC-P-5  | USD      | `5`                                          | 500       |
| AC-P-6  | USD      | `5.5`                                        | 550       |
| AC-P-7  | EUR      | `1.234,56 €`                                 | 123456    |
| AC-P-8  | EUR      | `1.234,56 €` (NBSP before €)                 | 123456    |
| AC-P-9  | EUR      | `1 234,56 €` (narrow no-break space group)   | 123456    |
| AC-P-10 | EUR      | `€1,234.56` (Ireland)                        | 123456    |
| AC-P-11 | EUR      | `€ 1.234,56` (Netherlands)                   | 123456    |
| AC-P-12 | EUR      | `0,99 €`                                     | 99        |
| AC-P-13 | EUR      | `1.000.000,00 €`                             | 100000000 |
| AC-P-14 | CAD      | `$1,234.56`                                  | 123456    |
| AC-P-15 | CAD      | `CA$1,234.56`                                | 123456    |
| AC-P-16 | CAD      | `1 234,56 $` (fr-CA)                         | 123456    |
| AC-P-17 | AUD      | `A$1,234.56`                                 | 123456    |
| AC-P-18 | AUD      | `AUD 12.30`                                  | 1230      |
| AC-P-19 | USD      | `-$5.00`                                     | -500      |
| AC-P-20 | USD      | `($5.00)`                                    | -500      |
| AC-P-21 | EUR      | `-1.234,56 €`                                | -123456   |
| AC-P-22 | EUR      | `1.234,56 €-`                                | -123456   |
| AC-P-23 | EUR      | `5,00-€`                                     | -500      |
| AC-P-24 | EUR      | `(€5,00)`                                    | -500      |
| AC-P-25 | EUR      | `−5,00 €` (U+2212 minus)                     | -500      |
| AC-P-26 | USD      | `＄１２．３０` (full-width chars)            | 1230      |
| AC-P-27 | EUR      | `1.234,56 -` (space before trailing minus)   | -123456   |
| AC-P-28 | EUR      | `1.234,56 - €` (space before minus + symbol) | -123456   |
| AC-P-29 | CAD      | `CAD$5.00`                                   | 500       |
| AC-P-30 | USD      | `US$5.00`                                    | 500       |
| AC-P-31 | CAD      | `$5.00 CAD`                                  | 500       |
| AC-P-32 | EUR      | `1234,56`                                    | 123456    |
| AC-P-33 | CAD      | `1234.56`                                    | 123456    |
| AC-P-34 | AUD      | `1234.56`                                    | 123456    |

Phase 1 baseline positive parsing is covered by:

- USD with indicator: AC-P-1 / AC-P-3; USD without indicator: AC-P-2.
- EUR with indicator: AC-P-7 / AC-P-10; EUR without indicator: AC-P-32.
- CAD with indicator: AC-P-14 / AC-P-15 / AC-P-16; CAD without indicator:
  AC-P-33.
- AUD with indicator: AC-P-17 / AC-P-18; AUD without indicator: AC-P-34.

Rows involving negative signs or accounting parentheses are Phase 2.

### AC-P-AMB · Parsing — ambiguity resolution (default options)

| #          | Currency | Input          | Expected           |
| ---------- | -------- | -------------- | ------------------ |
| AC-P-AMB-1 | USD      | `1,234`        | 123400 (grouping)  |
| AC-P-AMB-2 | EUR      | `1.234`        | 123400 (grouping)  |
| AC-P-AMB-3 | USD      | `1.23`         | 123 (decimal)      |
| AC-P-AMB-4 | EUR      | `1,23`         | 123 (decimal)      |
| AC-P-AMB-5 | USD      | `12.345`       | 1234500 (grouping) |
| AC-P-AMB-6 | EUR      | `1.234.567,89` | 123456789          |
| AC-P-AMB-7 | EUR      | `1.000`        | 100000 (grouping)  |

### AC-P-NEG · Parsing — negative (errors)

These rows are Phase 1 when they test basic malformed input or currency
contradiction; rows that depend on negative amount syntax, advanced grouping, or
post-MVP edge cases are Phase 2.

| #           | Currency | Input                          | Expected error            |
| ----------- | -------- | ------------------------------ | ------------------------- |
| AC-P-NEG-1  | USD      | `` (empty)                     | `EmptyInput`              |
| AC-P-NEG-2  | USD      | `abc`                          | `InvalidCharacter`        |
| AC-P-NEG-3  | EUR      | `$5.00`                        | `CurrencyMismatch`        |
| AC-P-NEG-4  | USD      | `€5,00`                        | `CurrencyMismatch`        |
| AC-P-NEG-5  | USD      | `1.2.3`                        | `InvalidGrouping`         |
| AC-P-NEG-6  | USD      | `1,23,456`                     | `InvalidGrouping`         |
| AC-P-NEG-7  | USD      | `12,34,567` (Indian)           | `InvalidGrouping`         |
| AC-P-NEG-8  | USD      | `1'234.56` (Swiss)             | `InvalidCharacter`        |
| AC-P-NEG-9  | USD      | `12.999` (>2 dp, reject mode)  | `TooManyFractionalDigits` |
| AC-P-NEG-10 | USD      | `99999999999999999999`         | `Overflow`                |
| AC-P-NEG-11 | USD      | `1,`                           | `MalformedNumber`         |
| AC-P-NEG-12 | USD      | `$1,234.56 USD`                | `MalformedCurrency`       |
| AC-P-NEG-13 | USD      | `US$ 12.34 USD`                | `MalformedCurrency`       |
| AC-P-NEG-14 | EUR      | `1.234,56 € EUR`               | `MalformedCurrency`       |
| AC-P-NEG-15 | USD      | `1.0000`                       | `InvalidGrouping`         |
| AC-P-NEG-16 | USD      | `1,2345`                       | `InvalidGrouping`         |
| AC-P-NEG-17 | USD      | `1,234.567,89`                 | `InvalidGrouping`         |
| AC-P-NEG-18 | USD      | `00`                           | `InvalidGrouping`         |
| AC-P-NEG-19 | USD      | `007`                          | `InvalidGrouping`         |
| AC-P-NEG-20 | CAD      | `US$5.00`                      | `CurrencyMismatch`        |
| AC-P-NEG-21 | USD      | input longer than 256 chars    | `InputTooLong`            |
| AC-P-NEG-22 | EUR      | `12 34,56 €` (bad space group) | `InvalidGrouping`         |
| AC-P-NEG-23 | USD      | `$`                            | `MalformedNumber`         |
| AC-P-NEG-24 | USD      | `USD`                          | `MalformedNumber`         |
| AC-P-NEG-25 | USD      | `-$`                           | `MalformedNumber`         |
| AC-P-NEG-26 | CAD      | `CAD$5.00 CAD`                 | `MalformedCurrency`       |

### AC-P-RND · Parsing — rounding mode enabled

Rounding-enabled parsing is Phase 2.

| #          | Currency | Input    | Mode      | Expected |
| ---------- | -------- | -------- | --------- | -------- |
| AC-P-RND-1 | USD      | `12.999` | HALF_UP   | 1300     |
| AC-P-RND-2 | USD      | `12.994` | HALF_UP   | 1299     |
| AC-P-RND-3 | USD      | `12.995` | HALF_EVEN | 1300     |
| AC-P-RND-4 | USD      | `12.985` | HALF_EVEN | 1298     |

### AC-P-ZERO · Parsing — zero normalization

Positive zero parsing is Phase 1. Negative zero normalization is Phase 2 because
negative syntax is Phase 2.

| #           | Currency | Input     | Expected         |
| ----------- | -------- | --------- | ---------------- |
| AC-P-ZERO-1 | USD      | `-$0.00`  | `Money(0, USD)`  |
| AC-P-ZERO-2 | USD      | `($0.00)` | `Money(0, USD)`  |
| AC-P-ZERO-3 | EUR      | `0,50 €`  | `Money(50, EUR)` |
| AC-P-ZERO-4 | USD      | `0.99`    | `Money(99, USD)` |

### AC-F · Formatting & round-trip

Formatting and format↔parse round-trip behavior are Phase 2.

| #       | Criterion                                                                                                                                                    |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| AC-F-1  | `format(Money(123456, USD), en-US)` = `Ok("$1,234.56")`.                                                                                                     |
| AC-F-2  | `format(Money(123456, EUR), de-DE)` = `Ok("1.234,56 €")`; `…, fr-FR` = `Ok("1 234,56 €")`; `…, en-IE` = `Ok("€1,234.56")`.                                   |
| AC-F-3  | **Round-trip:** for every supported `(currency, locale)`, if `format(m, locale)` returns `Ok(s)`, then `parse(s, currency)` = `m`.                           |
| AC-F-4  | Formatting emits `currency.exponent()` fractional digits; for current currencies that is exactly 2 (e.g. `Money(500, USD)` → `Ok("$5.00")`, not `Ok("$5")`). |
| AC-F-5  | `format(Money(-500, USD), en-US)` = `Ok("-$5.00")`.                                                                                                          |
| AC-F-6  | `format(Money(500, USD), unsupported-locale)` → `UnsupportedLocale`.                                                                                         |
| AC-F-7  | `format(Money(123456, CAD), en-CA)` = `Ok("CA$1,234.56")`; `…, fr-CA` = `Ok("1 234,56 $")`.                                                                  |
| AC-F-8  | `format(Money(123456, AUD), en-AU)` = `Ok("A$1,234.56")`.                                                                                                    |
| AC-F-9  | `format(Money(-123456, EUR), de-DE)` = `Ok("-1.234,56 €")`; `…, fr-FR` = `Ok("-1 234,56 €")`.                                                                |
| AC-F-10 | `format(Money(500, USD), "xx-XX")` from an external BCP-47 tag → `UnsupportedLocale`.                                                                        |
| AC-F-11 | `format(Money(0, USD), en-US)` = `Ok("$0.00")`.                                                                                                              |
| AC-F-12 | `format(neg(Money(0, USD)), en-US)` = `Ok("$0.00")` (zero is always signless).                                                                               |
| AC-F-13 | `format(Money(0, EUR), de-DE)` = `Ok("0,00 €")`.                                                                                                             |

### AC-A · Arithmetic & invariants

Structural equality/hash behavior is Phase 1. Monetary arithmetic, semantic
comparison, allocation, scaling, and total ordering are Phase 2.

| #       | Criterion                                                                                                                             |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| AC-A-1  | `add(Money(500,USD), Money(250,USD))` = `Money(750,USD)`.                                                                             |
| AC-A-2  | `add(Money(500,USD), Money(500,EUR))` → `CurrencyMismatch`.                                                                           |
| AC-A-3  | `mul_int(Money(199,USD), 3)` = `Money(597,USD)`.                                                                                      |
| AC-A-4  | `allocate(Money(1000,USD), 3)` = `[334,333,333]` USD, summing to `1000` (INV-5).                                                      |
| AC-A-5  | `add` that exceeds 64-bit range → `Overflow` (no wraparound).                                                                         |
| AC-A-6  | No API accepts or returns a floating-point amount (INV-2).                                                                            |
| AC-A-7  | `Money(500,USD) == Money(500,EUR)` is `false`; `try_cmp(Money(500,USD), Money(500,EUR))` → `CurrencyMismatch`.                        |
| AC-A-8  | `mul_ratio(Money(100,USD), 1, 0, HALF_UP)` → `DivisionByZero`.                                                                        |
| AC-A-9  | `allocate(Money(50,USD), 4)` = `[13,13,12,12]` USD, summing to `50` (INV-5).                                                          |
| AC-A-10 | `allocate(Money(2,USD), 4)` = `[1,1,0,0]` USD, summing to `2` (INV-5).                                                                |
| AC-A-11 | `allocate(Money(100,USD), 0)` → `InvalidArgument`.                                                                                    |
| AC-A-12 | `abs(Money(i64::MIN,USD))` → `Overflow`.                                                                                              |
| AC-A-13 | `neg(Money(0,USD))` = `Money(0,USD)`.                                                                                                 |
| AC-A-14 | `mul_ratio(Money(-25,USD), 1, 2, DOWN)` = `Money(-12,USD)`.                                                                           |
| AC-A-15 | `mul_ratio(Money(-25,USD), 1, 2, FLOOR)` = `Money(-13,USD)`.                                                                          |
| AC-A-16 | `mul_ratio(Money(-1,USD), 1, 2, HALF_UP)` = `Money(-1,USD)`.                                                                          |
| AC-A-17 | `allocate(Money(i64::MIN,USD), 3)` returns parts that sum exactly to `i64::MIN` or returns `Overflow`; it never panics.               |
| AC-A-18 | `from_major(-12, -34, USD)` = `Money(-1234,USD)`; `from_major(0, -34, USD)` = `Money(-34,USD)`.                                       |
| AC-A-19 | `from_major(12, -34, USD)` → `InvalidArgument`; `from_major(12, 100, USD)` → `InvalidArgument`.                                       |
| AC-A-20 | `mul_int(Money(500,USD), 0)` = `Money(0,USD)`.                                                                                        |
| AC-A-21 | `mul_int(Money(500,USD), -2)` = `Money(-1000,USD)`.                                                                                   |
| AC-A-22 | `mul_int(Money(-500,USD), -1)` = `Money(500,USD)`.                                                                                    |
| AC-A-23 | `mul_int(Money(0,USD), 100)` = `Money(0,USD)`.                                                                                        |
| AC-A-24 | `sub(Money(500,USD), Money(250,USD))` = `Money(250,USD)`.                                                                             |
| AC-A-25 | `sub(Money(500,USD), Money(500,EUR))` → `CurrencyMismatch`.                                                                           |
| AC-A-26 | `sub(Money(i64::MIN,USD), Money(1,USD))` → `Overflow` (no wraparound).                                                                |
| AC-A-27 | `min(Money(100,USD), Money(200,USD))` = `Money(100,USD)`.                                                                             |
| AC-A-28 | `max(Money(100,USD), Money(200,USD))` = `Money(200,USD)`.                                                                             |
| AC-A-29 | `min(Money(100,USD), Money(50,EUR))` → `CurrencyMismatch`.                                                                            |
| AC-A-30 | `mul_ratio(Money(500,USD), 0, 1, HALF_UP)` = `Money(0,USD)`.                                                                          |
| AC-A-31 | `mul_ratio(Money(500,USD), 0, 0, HALF_UP)` = `Money(0,USD)` (zero numerator short-circuits `DivisionByZero`).                         |
| AC-A-32 | `mul_ratio(Money(100,USD), num, 1, HALF_UP)` with `num` outside signed 64-bit range → `Overflow`.                                     |
| AC-A-33 | `mul_ratio(Money(100,USD), 1, -2, HALF_UP)` = `Money(-50,USD)` (negative denominator; sign follows signed ratio).                     |
| AC-A-34 | `cmp(Money(100,USD), Money(200,USD))` = `Less` (same-currency total order by amount).                                                 |
| AC-A-35 | `cmp(Money(100,USD), Money(999999,AUD))` = `Greater` (USD orders after AUD by ISO 4217 alpha code; amount ignored across currencies). |

### AC-S · Serialization

Serialization/deserialization are Phase 1.

| #       | Criterion                                                                                                                        |
| ------- | -------------------------------------------------------------------------------------------------------------------------------- |
| AC-S-1  | `Money(123456,USD)` serializes as `{ "amount_minor": "123456", "currency": "USD" }`.                                             |
| AC-S-2  | `Money(9007199254740993,USD)` serializes and deserializes exactly with string `amount_minor`; no JSON number conversion is used. |
| AC-S-3  | Deserializing `{ "amount_minor": "123456", "currency": "XYZ" }` → `UnknownCurrency`.                                             |
| AC-S-4  | Deserializing `{ "amount_minor": 123456, "currency": "USD" }` (JSON number) → `MalformedWireValue` (not coerced).                |
| AC-S-5  | Deserializing `{ "amount_minor": "12.5", "currency": "USD" }` (non-integer) → `InvalidAmountMinor`.                              |
| AC-S-6  | Deserializing `{ "amount_minor": "007", "currency": "USD" }` (leading zero) → `InvalidAmountMinor`.                              |
| AC-S-7  | Deserializing `{ "amount_minor": "99999999999999999999", "currency": "USD" }` (out of range) → `AmountOutOfRange`.               |
| AC-S-8  | Deserializing `{ "amount_minor": "+5", "currency": "USD" }` (leading plus) → `InvalidAmountMinor`.                               |
| AC-S-9  | Deserializing `{ "amount_minor": "", "currency": "USD" }` (empty string) → `InvalidAmountMinor`.                                 |
| AC-S-10 | Deserializing `{ "amount_minor": " 5 ", "currency": "USD" }` (surrounding whitespace) → `InvalidAmountMinor`.                    |

### AC-NFR · Non-functional requirements

AC-NFR-1 and AC-NFR-3 apply from Phase 1. AC-NFR-2 applies when formatting ships
in Phase 2.

| #        | Criterion                                                                    |
| -------- | ---------------------------------------------------------------------------- |
| AC-NFR-1 | Repeating `parse` with the same input and options returns identical results. |
| AC-NFR-2 | Repeating `format` with the same money and locale returns identical results. |
| AC-NFR-3 | Operations return new `Money` values and do not mutate their inputs.         |

---

## 4. Decision status

These design decisions were once open; the spec now records their status. Items
marked `decided` are normative for the phase that contains them. Items marked
`phase-3 backlog` are not committed for delivery until product or architecture
decisions are made.

| #   | Decision                                                      | Status            | Rationale / where it closes                                                                                                                                                                                                                                              |
| --- | ------------------------------------------------------------- | ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 1   | Signed vs unsigned storage                                    | `decided`         | Signed (BR-2, §2.1) to support refunds/discounts/deltas; price-level callers enforce non-negativity, not `Money`.                                                                                                                                                        |
| 2   | Parsing default for extra fractional digits                   | `decided`         | Reject by default (`TooManyFractionalDigits`, §2.5); callers opt into a rounding mode per call so scraped data is never silently corrupted. Broader messy-input policy remains a Phase 3 topic.                                                                          |
| 3   | Single-separator 3-trailing-digit ambiguity (`1.234`/`1,234`) | `decided`         | Treated as grouping (§2.4 step 6); the only other reading (3-dp decimal) is invalid for 2-dp currencies, so the risk is nil.                                                                                                                                             |
| 4   | Rendering/formatting ownership                                | `phase-3 backlog` | This spec currently defines formatting as a Phase 2 library capability (§2.8) for completeness and round-trip testing. Whether ownership should move to the presentation layer is explicitly a Phase 3 discussion topic.                                                 |
| 5   | Currency auto-detection from the string alone                 | `phase-3 backlog` | The caller must supply the expected currency because `$` is ambiguous (§2.4 step 3). Symbol-only inference is not part of Phase 1 or Phase 2.                                                                                                                            |
| 6   | JSON wire format for `amount_minor`                           | `decided`         | String-encoded `amount_minor`, **v1** (§2.10), to preserve exactness through double-backed JSON parsers; JSON-number form is rejected (no prior deployed consumer to stay compatible with). If compatibility evidence appears, migration policy becomes a Phase 3 topic. |
| 7   | Currency expansion beyond USD/EUR/CAD/AUD                     | `phase-3 backlog` | The current design is exponent-driven, but adding zero- or three-exponent currencies is not committed for Phase 1 or Phase 2.                                                                                                                                            |
| 8   | FX, tax/VAT, and cross-system multi-currency strategy         | `phase-3 backlog` | Explicitly out of scope for `libs/money`; handled by roadmap/product architecture before any future spec change.                                                                                                                                                         |

---

## Implementation notes (non-normative)

Mapping to `libs/money` (Rust): `amount_minor` → `i64`; `Currency` → a
`#[non_exhaustive]`-friendly enum; arithmetic via `checked_add`/`checked_mul`
returning `Result`; `ParseError`/`MoneyError`/`DeserializeError` as `enum`s
implementing `std::error::Error`; `Money` serializes/deserializes through serde
using the canonical wire format in §2.10 (string `amount_minor`); no `f64` in
the public API. Locale/currency tables and the step-0 fold map as `const` data
so normalization does not depend on the runtime's Unicode version.

`Money` implements `Debug` for developer output, e.g. `Money { amount_minor:
123456, currency: USD }`, but does not implement `Display`. Human-readable
output must go through `format(money, locale)` so callers choose the intended
locale explicitly. `PartialEq`/`Eq`/`Hash` and `Ord` are total over
`(amount_minor, currency)` (currencies ordered by ISO alpha code); semantic
amount comparison uses `try_cmp`.
