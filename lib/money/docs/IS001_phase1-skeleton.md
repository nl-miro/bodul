# Money — Phase 1 Skeleton (Implementation Spec)

Status: Draft · Version: 0.1.0 · Scope: `lib/money` (Rust crate) · Phase 1 only

This is an **implementation specification**. It defines the *first* deliverable of
the `money` crate: a **typed skeleton** in which the Phase 1 public interface
exists as real Rust types and function signatures, but **every function body is
`todo!()`**. The crate compiles and exposes the contract; it implements no
behaviour.

It is derived from and subordinate to:

- [TS001 — Money Specification](./TS001_money-type.md) (authoritative behaviour).
- [BR001 — Business Requirements](./BR001_initial-business-requirements.md).

If this document and TS001 disagree on a signature or type, **TS001 governs** and
this document must be corrected.

> **Crate-path note.** TS001 names the crate `libs/money` throughout. The crate
> actually lives at **`lib/money/`**, next to these docs. This is a known
> discrepancy; aligning the spec text to `lib/money` is a follow-up and is out of
> scope for the skeleton.

---

## 1. Intent: skeleton first

Before any monetary behaviour is written, the crate must first exist as a
**signature-only skeleton**:

- The full Phase 1 public surface (TS001 §1.4, §2.9, §2.7) is present as Rust
  types and function signatures — the **inputs and outputs are fixed**.
- **Every function body is `todo!()`.** No parsing, arithmetic, or serialization
  logic is implemented at this stage.
- The crate **must compile** (`cargo build`) and its public API **must render**
  (`cargo doc`). `todo!()` satisfies any return type, so a signature-complete
  skeleton builds cleanly.
- No tests are added yet — there is no behaviour to test. Tests arrive as each
  stub is filled in against the TS001 acceptance criteria.

This pins the contract down first and gives implementers a compiling shell to
fill in, one stub at a time, without re-litigating the public shape.

---

## 2. Crate layout

```
lib/money/
├── Cargo.toml          # package `money`, edition 2021, no dependencies
├── docs/               # BR001, TS001, IS001 (this doc)
└── src/
    ├── lib.rs          # crate docs, module decls, public re-exports
    ├── currency.rs     # Currency enum + exponent
    ├── money.rs        # Money struct, constructors, accessors
    ├── parse.rs        # parse, ParseOptions, RoundingMode
    ├── serialize.rs    # serialize / deserialize (canonical §2.10 wire format)
    └── error.rs        # ParseError, MoneyError, DeserializeError
```

- **No external dependencies** in the skeleton. TS001's implementation notes call
  for serde integration; that is deferred. `serialize`/`deserialize` are declared
  as hand-rolled functions for now and stubbed with `todo!()`.
- `Money` and `Currency` derive `Debug, Clone, Copy, PartialEq, Eq, Hash`.
  Equality/hashing are total over the whole `(amount_minor, currency)` value
  (TS001 §2.6, INV-3).
- **No `Display` on `Money`** — TS001 implementation notes require human-readable
  output to go through the Phase 2 `format(money, locale)` function, so `Display`
  is deliberately not implemented.
- `Money` fields are **private**; values are created only via the constructors and
  inspected only via the accessors (TS001 §2.1).

---

## 3. Phase 1 public surface (the contract)

Reproduced from TS001 §2.9 (Phase 1 subset) and §2.7. Rust shape uses inherent
methods / associated functions rather than free functions. Every body is
`todo!()`.

### Types

```rust
pub enum Currency { USD, EUR, CAD, AUD }

pub struct Money { /* private: amount_minor: i64, currency: Currency */ }

pub struct ParseOptions { pub rounding: Option<RoundingMode> } // Default: None

pub enum RoundingMode { HalfUp, HalfEven, Down, Up, Ceiling, Floor }
```

`RoundingMode` is defined now because `ParseOptions` references it, but its
*behaviour* (rounding-enabled parsing) is Phase 2.

### Functions (signatures only)

```rust
impl Currency {
    pub fn exponent(self) -> u8;                 // TS001 §2.1/§2.2: 2 for all current currencies
}

impl Money {
    pub fn new(amount_minor: i64, currency: Currency) -> Money;              // infallible
    pub fn from_major(units: i64, fractional_minor: i64, currency: Currency)
        -> Result<Money, MoneyError>;            // §2.9: sign/magnitude rules, overflow-checked
    pub fn minor_units(&self) -> i64;
    pub fn currency(&self) -> Currency;

    pub fn parse(raw: &str, currency: Currency, options: ParseOptions)
        -> Result<Money, ParseError>;            // §2.4 algorithm (Phase 1: positive baseline)

    pub fn serialize(&self) -> String;           // §2.10 canonical wire format
    pub fn deserialize(wire: &str) -> Result<Money, DeserializeError>;  // §2.10, NOT the §2.4 parser
}
```

### Error enums (TS001 §2.7)

- `ParseError` — `EmptyInput`, `InputTooLong`, `InvalidCharacter`, `MalformedSign`,
  `MalformedCurrency`, `CurrencyMismatch`, `MalformedNumber`, `InvalidGrouping`,
  `TooManyFractionalDigits`, `Overflow`. The **full** enum is defined in Phase 1
  even though the sign/parentheses variants only become reachable in Phase 2
  (TS001 §1.4).
- `MoneyError` — `CurrencyMismatch`, `Overflow`, `DivisionByZero`,
  `InvalidArgument`. Needed by `from_major` (`InvalidArgument`, `Overflow`).
- `DeserializeError` — `UnknownCurrency`, `InvalidAmountMinor`, `AmountOutOfRange`,
  `MalformedWireValue`.

Each error enum derives `Debug, Clone, PartialEq, Eq` and implements `Display` +
`std::error::Error`.

---

## 4. Out of scope (Phase 2 — not in the skeleton)

The following TS001 surface is **omitted** from the skeleton and added in Phase 2:

- `format(value, locale?)`, the `Locale` enum, and `FormatError`.
- Arithmetic & comparison: `add`, `sub`, `neg`, `abs`, `mul_int`, `mul_ratio`,
  `allocate`, `try_cmp`, `cmp`, `min`, `max`.
- Rounding *behaviour* (the `RoundingMode` type exists, but no rounding is applied).

---

## 5. Traceability

Each stub will eventually satisfy specific TS001 acceptance criteria. The skeleton
references these by ID (it does not duplicate them). Phase 1 targets:

| Stub                       | TS001 acceptance criteria (Phase 1)                       |
| -------------------------- | --------------------------------------------------------- |
| `Money::new` / accessors   | underpin AC-A structural equality/hash rows               |
| `Currency::exponent`       | §2.2 (returns 2); drives parse fraction handling          |
| `Money::from_major`        | AC-A-18, AC-A-19                                           |
| `Money::parse`             | AC-P baseline positives, AC-P-AMB, AC-P-NEG, AC-P-ZERO-3/4 |
| `Money::serialize`/`deserialize` | AC-S-1 … AC-S-10                                     |
| crate-wide                 | AC-NFR-1 (determinism), AC-NFR-3 (no mutation)            |

Phase 2 acceptance criteria (AC-F, the arithmetic rows of AC-A, AC-P-RND, negative
AC-P rows) are not covered by the skeleton.

---

## 6. Definition of done (skeleton)

- All Section 3 types and signatures exist in `lib/money/src/`.
- Every function body is `todo!()`; no behaviour is implemented.
- `cargo build` succeeds; `cargo doc --no-deps` renders the public surface.
- `Money` has no `Display` impl; `Money` fields are private.
- No Phase 2 symbol (Section 4) is present.
