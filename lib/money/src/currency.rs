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
        // All four supported currencies are exponent-2 (TS001 §2.2). The match is
        // exhaustive so adding a non-2 currency later forces this to be revisited.
        match self {
            Currency::USD | Currency::EUR | Currency::CAD | Currency::AUD => 2,
        }
    }

    /// ISO 4217 alpha code for this currency, used by the canonical wire format
    /// (TS001 §2.10) and currency-indicator matching (§2.4).
    pub(crate) fn iso_code(self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
        }
    }

    /// Resolve an ISO 4217 alpha code to a supported [`Currency`], or `None` if it
    /// is not one of the four (TS001 §2.2). Case-sensitive: the canonical wire
    /// format uses upper-case codes.
    pub(crate) fn from_iso_code(code: &str) -> Option<Currency> {
        match code {
            "USD" => Some(Currency::USD),
            "EUR" => Some(Currency::EUR),
            "CAD" => Some(Currency::CAD),
            "AUD" => Some(Currency::AUD),
            _ => None,
        }
    }
}
