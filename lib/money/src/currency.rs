//! Supported currencies (TS001 §2.2).

/// The currencies supported by this library. All four have a minor-unit exponent
/// of `2` (100 minor units = 1 major unit), so "cents" is uniform across them
/// (TS001 §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Currency {
    /// US Dollar (ISO 840).
    USD,
    /// Euro (ISO 978).
    EUR,
    /// Canadian Dollar (ISO 124).
    CAD,
    /// Australian Dollar (ISO 036).
    AUD,
}

impl Currency {
    /// Number of fractional (minor-unit) digits for this currency — the source of
    /// truth for fraction handling in parsing and formatting (TS001 §2.1/§2.2).
    /// Returns `2` for every currently supported currency.
    pub fn exponent(self) -> u8 {
        // TODO(phase-1): return the minor-unit exponent (2 for USD/EUR/CAD/AUD).
        todo!("Currency::exponent — TS001 §2.2")
    }
}
