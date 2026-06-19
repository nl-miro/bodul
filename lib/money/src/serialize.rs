//! Canonical serialization to/from the wire format (TS001 §2.10).
//!
//! The wire format is the raw data model, e.g.
//! `{ "amount_minor": "123456", "currency": "USD" }`, where `amount_minor` is a
//! base-10 **string** (grammar `-?[0-9]+`, no leading `+`, no leading zeros
//! except a literal `0`, `-0` rejected) so values round-trip exactly through
//! double-backed JSON parsers.
//!
//! These functions are **exact** and MUST NOT use the free-form §2.4 parser.

use crate::error::DeserializeError;
use crate::money::Money;

impl Money {
    /// Serialize to the canonical v1 wire format (TS001 §2.10). Exact; never lossy.
    /// (AC-S-1, AC-S-2.)
    pub fn serialize(&self) -> String {
        // TODO(phase-1): emit { "amount_minor": "<i64>", "currency": "<ISO>" }.
        todo!("Money::serialize — TS001 §2.10")
    }

    /// Deserialize from the canonical v1 wire format (TS001 §2.10).
    ///
    /// Rejects a JSON-number `amount_minor` as [`DeserializeError::MalformedWireValue`]
    /// (not coerced), a grammar-violating string as [`DeserializeError::InvalidAmountMinor`],
    /// an out-of-range integer as [`DeserializeError::AmountOutOfRange`], and an
    /// unknown currency code as [`DeserializeError::UnknownCurrency`]. Exact; MUST
    /// NOT use the §2.4 parser. (AC-S-3 … AC-S-10.)
    pub fn deserialize(wire: &str) -> Result<Money, DeserializeError> {
        // TODO(phase-1): parse the canonical wire object exactly per §2.10.
        let _ = wire;
        todo!("Money::deserialize — TS001 §2.10")
    }
}
