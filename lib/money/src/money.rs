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
        Money {
            amount_minor,
            currency,
        }
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
        let scale = 10_i64.pow(currency.exponent() as u32);

        // `fractional_minor` is a sub-major remainder, so its magnitude must be
        // strictly less than one major unit (TS001 §2.9; AC-A-19 `12,100`).
        if fractional_minor.abs() >= scale {
            return Err(MoneyError::InvalidArgument);
        }

        // `units` and `fractional_minor` must agree in sign unless either is zero,
        // so `from_major(0, -34)` is allowed but `from_major(12, -34)` is not
        // (TS001 §2.9; AC-A-18, AC-A-19).
        let mixed_signs = (units > 0 && fractional_minor < 0)
            || (units < 0 && fractional_minor > 0);
        if mixed_signs {
            return Err(MoneyError::InvalidArgument);
        }

        // Overflow-checked assembly: units * 10^e + fractional_minor (NFR-2).
        let amount_minor = units
            .checked_mul(scale)
            .and_then(|major| major.checked_add(fractional_minor))
            .ok_or(MoneyError::Overflow)?;

        Ok(Money::new(amount_minor, currency))
    }

    /// The signed minor-unit (cents) count.
    pub fn minor_units(&self) -> i64 {
        self.amount_minor
    }

    /// The currency of this value.
    pub fn currency(&self) -> Currency {
        self.currency
    }
}
