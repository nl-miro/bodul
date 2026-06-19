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
        match self {
            Currency::USD | Currency::EUR | Currency::CAD | Currency::AUD => 2,
        }
    }

    pub(crate) fn iso_code(self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
        }
    }

    pub(crate) fn from_iso_code(code: &str) -> Option<Self> {
        match code {
            "USD" => Some(Currency::USD),
            "EUR" => Some(Currency::EUR),
            "CAD" => Some(Currency::CAD),
            "AUD" => Some(Currency::AUD),
            _ => None,
        }
    }
}
