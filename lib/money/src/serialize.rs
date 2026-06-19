//! Canonical serialization to/from the wire format (TS001 §2.10).
//!
//! The wire format is the raw data model, e.g.
//! `{ "amount_minor": "123456", "currency": "USD" }`, where `amount_minor` is a
//! base-10 **string** (grammar `-?[0-9]+`, no leading `+`, no leading zeros
//! except a literal `0`, `-0` rejected) so values round-trip exactly through
//! double-backed JSON parsers.
//!
//! These functions are **exact** and MUST NOT use the free-form §2.4 parser.

use crate::currency::Currency;
use crate::error::DeserializeError;
use crate::money::Money;

impl Money {
    /// Serialize to the canonical v1 wire format (TS001 §2.10). Exact; never lossy.
    /// (AC-S-1, AC-S-2.)
    pub fn serialize(&self) -> String {
        format!(
            r#"{{"amount_minor":"{}","currency":"{}"}}"#,
            self.minor_units(),
            self.currency().iso_code()
        )
    }

    /// Deserialize from the canonical v1 wire format (TS001 §2.10).
    ///
    /// Rejects a JSON-number `amount_minor` as [`DeserializeError::MalformedWireValue`]
    /// (not coerced), a grammar-violating string as [`DeserializeError::InvalidAmountMinor`],
    /// an out-of-range integer as [`DeserializeError::AmountOutOfRange`], and an
    /// unknown currency code as [`DeserializeError::UnknownCurrency`]. Exact; MUST
    /// NOT use the §2.4 parser. (AC-S-3 … AC-S-10.)
    pub fn deserialize(wire: &str) -> Result<Money, DeserializeError> {
        let wire = wire.trim();

        // Must be a JSON object: starts with `{` and ends with `}`
        if !wire.starts_with('{') || !wire.ends_with('}') {
            return Err(DeserializeError::MalformedWireValue);
        }

        let inner = wire[1..wire.len() - 1].trim();
        let bytes = inner.as_bytes();
        let mut pos = 0;
        let mut amount_minor_str: Option<&str> = None;
        let mut currency_str: Option<&str> = None;
        let mut first = true;

        loop {
            pos = skip_whitespace(bytes, pos);

            // End of object
            if pos >= bytes.len() {
                break;
            }

            // Comma between fields
            if !first {
                if pos >= bytes.len() || bytes[pos] != b',' {
                    return Err(DeserializeError::MalformedWireValue);
                }
                pos += 1;
                pos = skip_whitespace(bytes, pos);
            }
            first = false;

            // Expect '"' for key
            if pos >= bytes.len() || bytes[pos] != b'"' {
                return Err(DeserializeError::MalformedWireValue);
            }
            pos += 1;

            // Read key name
            let key_start = pos;
            while pos < bytes.len() && bytes[pos] != b'"' {
                if bytes[pos] == b'\\' {
                    return Err(DeserializeError::MalformedWireValue);
                }
                pos += 1;
            }
            if pos >= bytes.len() {
                return Err(DeserializeError::MalformedWireValue);
            }
            let key = std::str::from_utf8(&bytes[key_start..pos])
                .map_err(|_| DeserializeError::MalformedWireValue)?;
            pos += 1; // skip closing '"'

            // Expect ':'
            pos = skip_whitespace(bytes, pos);
            if pos >= bytes.len() || bytes[pos] != b':' {
                return Err(DeserializeError::MalformedWireValue);
            }
            pos += 1;

            // Skip whitespace before value
            pos = skip_whitespace(bytes, pos);

            match key {
                "amount_minor" => {
                    if amount_minor_str.is_some() {
                        return Err(DeserializeError::MalformedWireValue); // duplicate key
                    }
                    if pos >= bytes.len() || bytes[pos] != b'"' {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                    pos += 1;
                    let val_start = pos;
                    while pos < bytes.len() && bytes[pos] != b'"' {
                        if bytes[pos] == b'\\' {
                            return Err(DeserializeError::MalformedWireValue);
                        }
                        pos += 1;
                    }
                    if pos >= bytes.len() {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                    let val = std::str::from_utf8(&bytes[val_start..pos])
                        .map_err(|_| DeserializeError::MalformedWireValue)?;
                    pos += 1;
                    amount_minor_str = Some(val);
                }
                "currency" => {
                    if currency_str.is_some() {
                        return Err(DeserializeError::MalformedWireValue); // duplicate key
                    }
                    if pos >= bytes.len() || bytes[pos] != b'"' {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                    pos += 1;
                    let val_start = pos;
                    while pos < bytes.len() && bytes[pos] != b'"' {
                        if bytes[pos] == b'\\' {
                            return Err(DeserializeError::MalformedWireValue);
                        }
                        pos += 1;
                    }
                    if pos >= bytes.len() {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                    let val = std::str::from_utf8(&bytes[val_start..pos])
                        .map_err(|_| DeserializeError::MalformedWireValue)?;
                    pos += 1;
                    currency_str = Some(val);
                }
                _ => {
                    // Unknown key — reject; canonical v1 only knows two fields
                    return Err(DeserializeError::MalformedWireValue);
                }
            }
        }

        let amount_str = amount_minor_str.ok_or(DeserializeError::MalformedWireValue)?;
        let curr_str = currency_str.ok_or(DeserializeError::MalformedWireValue)?;

        // Validate amount_minor string grammar (TS001 §2.10):
        // Grammar: -?[0-9]+ , no leading +, no leading zeros except literal "0", no "-0"
        validate_amount_minor_string(amount_str)?;

        // Parse the integer
        let amount: i64 = amount_str
            .parse()
            .map_err(|_| DeserializeError::AmountOutOfRange)?;

        // Parse currency
        let currency =
            Currency::from_iso_code(curr_str).ok_or(DeserializeError::UnknownCurrency)?;

        Ok(Money::new(amount, currency))
    }
}

fn skip_whitespace(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len()
        && (bytes[pos] == b' ' || bytes[pos] == b'\t' || bytes[pos] == b'\n' || bytes[pos] == b'\r')
    {
        pos += 1;
    }
    pos
}

/// Validate the `amount_minor` string per TS001 §2.10 grammar:
/// - Must be `-?[0-9]+`
/// - No leading `+`
/// - No leading zeros except the single literal `0`
/// - `-0` is rejected (zero serializes as `"0"`)
/// - Non-empty, no surrounding/internal whitespace
fn validate_amount_minor_string(s: &str) -> Result<(), DeserializeError> {
    if s.is_empty() {
        return Err(DeserializeError::InvalidAmountMinor);
    }

    // Check for whitespace
    if s.chars().any(|c| c.is_whitespace()) {
        return Err(DeserializeError::InvalidAmountMinor);
    }

    let bytes = s.as_bytes();
    let mut pos = 0;

    // Optional leading minus
    let negative = bytes[pos] == b'-';
    if negative {
        pos += 1;
    }

    // Must have at least one digit
    if pos >= bytes.len() || !bytes[pos].is_ascii_digit() {
        return Err(DeserializeError::InvalidAmountMinor);
    }

    // Leading zero check
    if bytes[pos] == b'0' {
        if pos + 1 < bytes.len() {
            // Leading zero with more digits → invalid (e.g. "007", "0", "-0")
            return Err(DeserializeError::InvalidAmountMinor);
        }
        if negative {
            // "-0" is invalid
            return Err(DeserializeError::InvalidAmountMinor);
        }
        return Ok(());
    }

    // All remaining characters must be ASCII digits
    for &b in &bytes[pos..] {
        if !b.is_ascii_digit() {
            return Err(DeserializeError::InvalidAmountMinor);
        }
    }

    Ok(())
}
