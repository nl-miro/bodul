//! # money
//!
//! Exact, currency-aware money value type for USD, EUR, CAD, and AUD.
//!
//! This crate implements the Phase 1 scrape → parse → store path: exact
//! minor-unit storage, construction/accessors, positive price parsing, and
//! canonical wire serialization.
//!
//! See the specs under `docs/`:
//! - `TS001_money-type.md` — authoritative behaviour (governs on any conflict).
//! - `IS001_phase1-skeleton.md` — this skeleton's contract and scope.
//! - `BR001_initial-business-requirements.md` — business context.
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
