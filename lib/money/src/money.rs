//! The `Money` value type, constructors, and accessors (TS001 §2.1, §2.9).

use crate::currency::Currency;
use crate::error::MoneyError;

/// An exact monetary amount paired with its currency.
///
/// Stored as a signed 64-bit count of **minor units** (cents); no floating point
/// (TS001 §2.1, INV-2). Fields are private — values are created through the
/// constructors and inspected through the accessors, so callers cannot bypass
/// invariants by raw struct construction.
///
/// Equality, hashing, and ordering are total over the whole `(amount_minor,
/// currency)` value, so `Money(500, USD) != Money(500, EUR)` and `Money` is safe
/// as a hash key (TS001 §2.6, INV-3). `Display` is intentionally **not**
/// implemented; human-readable output must go through the Phase 2
/// `format(money, locale)` function.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Money {
    amount_minor: i64,
    currency: Currency,
}

impl Money {
    /// Construct a money value directly from a signed minor-unit count.
    ///
    /// Infallible: every `i64` paired with a supported currency is a valid
    /// `Money` (TS001 §2.9).
    pub fn new(amount_minor: i64, currency: Currency) -> Money {
        // TODO(phase-1): construct Money { amount_minor, currency }.
        let _ = (amount_minor, currency);
        todo!("Money::new — TS001 §2.9")
    }

    /// Combine whole major units and a signed fractional minor-unit component
    /// using `currency.exponent()`.
    ///
    /// `fractional_minor` must have magnitude less than `10^exponent`, and `units`
    /// and `fractional_minor` must share a sign unless either is zero; otherwise
    /// returns [`MoneyError::InvalidArgument`]. Overflow-checked. (TS001 §2.9;
    /// AC-A-18, AC-A-19.)
    pub fn from_major(
        units: i64,
        fractional_minor: i64,
        currency: Currency,
    ) -> Result<Money, MoneyError> {
        // TODO(phase-1): validate sign/magnitude, assemble minor units (overflow-checked).
        let _ = (units, fractional_minor, currency);
        todo!("Money::from_major — TS001 §2.9")
    }

    /// The signed minor-unit (cents) count.
    pub fn minor_units(&self) -> i64 {
        // TODO(phase-1): return self.amount_minor.
        todo!("Money::minor_units — TS001 §2.9")
    }

    /// The currency of this value.
    pub fn currency(&self) -> Currency {
        // TODO(phase-1): return self.currency.
        todo!("Money::currency — TS001 §2.9")
    }
}
