//! Supported currencies (TS001 §2.2).

/// The currencies supported by this library. All four have a minor-unit exponent
/// of `2` (100 minor units = 1 major unit), so "cents" is uniform across them
/// (TS001 §2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Currency {
    /// Australian Dollar (ISO 036).
    AUD,
    /// Canadian Dollar (ISO 124).
    CAD,
    /// Euro (ISO 978).
    EUR,
    /// US Dollar (ISO 840).
    USD,
}

/// All known currency indicator strings with their associated currency, sorted
/// by length descending so the parser matches longest-first (TS001 §2.4 step 3).
/// `$` is ambiguous (USD, CAD, AUD) and receives special handling.
pub(crate) const ALL_INDICATORS: &[(&str, Currency)] = &[
    ("CAD$", Currency::CAD),
    ("Can$", Currency::CAD),
    ("AUD$", Currency::AUD),
    ("CA$", Currency::CAD),
    ("AU$", Currency::AUD),
    ("US$", Currency::USD),
    ("USD", Currency::USD),
    ("EUR", Currency::EUR),
    ("CAD", Currency::CAD),
    ("AUD", Currency::AUD),
    ("A$", Currency::AUD),
    ("C$", Currency::CAD),
    ("$", Currency::USD), // ambiguous — special handling
    ("\u{20AC}", Currency::EUR),
];

impl Currency {
    /// Number of fractional (minor-unit) digits for this currency — the source of
    /// truth for fraction handling in parsing and formatting (TS001 §2.1/§2.2).
    /// Returns `2` for every currently supported currency.
    pub fn exponent(self) -> u8 {
        2
    }

    /// ISO 4217 alpha code for this currency.
    pub(crate) fn iso_code(self) -> &'static str {
        match self {
            Currency::USD => "USD",
            Currency::EUR => "EUR",
            Currency::CAD => "CAD",
            Currency::AUD => "AUD",
        }
    }

    /// Parse an ISO 4217 alpha code into a `Currency`, case-sensitive exact match.
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
