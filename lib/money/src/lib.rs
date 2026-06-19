//! # money
//!
//! Exact, currency-aware money value type for USD, EUR, CAD, and AUD.
//!
//! **Phase 1 is implemented:** the value type with signed integer minor-unit
//! storage, construction/accessors, positive-amount parsing (TS001 §2.4 baseline),
//! and canonical serialize/deserialize (§2.10). Monetary arithmetic, locale-aware
//! formatting, negative-amount parsing, and rounding *behaviour* are Phase 2 and are
//! not yet present.
//!
//! See the specs under `docs/`:
//! - `TS001_money-type.md` — authoritative behaviour (governs on any conflict).
//! - `IS001_phase1-skeleton.md` — the Phase 1 contract and scope.
//! - `BR001_initial-business-requirements.md` — business context.
//!
//! Any place where Phase 1 deviates from a spec acceptance criterion is recorded in
//! `breakingchanges-claude.md` at the crate root.
//!
//! Amounts are stored as a signed integer count of **minor units** (cents); no
//! floating point ever touches an amount (TS001 §2.1, INV-2).

mod currency;
mod error;
mod money;
mod parse;
mod serialize;

pub use currency::Currency;
pub use error::{DeserializeError, MoneyError, ParseError};
pub use money::Money;
pub use parse::{ParseOptions, RoundingMode};
