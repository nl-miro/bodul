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
    ///
    /// `amount_minor` is emitted as a base-10 **string** so values beyond the JSON
    /// safe-integer range survive double-backed JSON parsers. The `i64` `Display`
    /// already matches the canonical grammar (`-?[0-9]+`, no leading zeros, `0` for
    /// zero), so no extra normalization is needed.
    pub fn serialize(&self) -> String {
        format!(
            r#"{{ "amount_minor": "{}", "currency": "{}" }}"#,
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
        // Structural read: a single JSON object with `amount_minor` and `currency`.
        // Any structural problem (not an object, missing field, non-string value) is
        // `MalformedWireValue` (AC-S-4).
        let fields = scan_object(wire).ok_or(DeserializeError::MalformedWireValue)?;
        let amount_value = fields
            .amount_minor
            .ok_or(DeserializeError::MalformedWireValue)?;
        let currency_value = fields.currency.ok_or(DeserializeError::MalformedWireValue)?;

        // `amount_minor` MUST be a JSON string; a number form is rejected, never
        // coerced (TS001 §2.10; AC-S-4).
        let amount_str = match amount_value {
            WireValue::Str(s) => s,
            WireValue::NonStr => return Err(DeserializeError::MalformedWireValue),
        };
        let currency_str = match currency_value {
            WireValue::Str(s) => s,
            WireValue::NonStr => return Err(DeserializeError::MalformedWireValue),
        };

        // Unknown ISO code (AC-S-3).
        let currency =
            Currency::from_iso_code(&currency_str).ok_or(DeserializeError::UnknownCurrency)?;

        // Canonical grammar before range (AC-S-5/6/8/9/10), then range (AC-S-7).
        if !is_canonical_amount(&amount_str) {
            return Err(DeserializeError::InvalidAmountMinor);
        }
        let amount_minor = amount_str
            .parse::<i64>()
            .map_err(|_| DeserializeError::AmountOutOfRange)?;

        Ok(Money::new(amount_minor, currency))
    }
}

/// Canonical `amount_minor` grammar (TS001 §2.10): `-?[0-9]+` with no leading `+`,
/// no leading zeros except the literal `0`, `-0` rejected, no whitespace, non-empty.
fn is_canonical_amount(s: &str) -> bool {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    // No leading zeros except a lone `0`; and `-0` is not canonical.
    if digits.len() > 1 && digits.starts_with('0') {
        return false;
    }
    if negative && digits == "0" {
        return false;
    }
    true
}

/// A wire value, distinguishing a JSON string from anything else (number, bool,
/// null, …). Phase 1 only needs the string/non-string distinction and the string
/// contents.
enum WireValue {
    Str(String),
    NonStr,
}

/// The two fields we care about, each present or absent.
struct Fields {
    amount_minor: Option<WireValue>,
    currency: Option<WireValue>,
}

/// Minimal hand-rolled JSON object scanner (no serde dependency — TS001 §2.10 /
/// IS001 keep the skeleton dependency-free). Returns `None` on any structural
/// malformation. Unknown keys are ignored.
fn scan_object(wire: &str) -> Option<Fields> {
    let mut sc = Scanner::new(wire);
    sc.skip_ws();
    if sc.next()? != '{' {
        return None;
    }

    let mut amount_minor = None;
    let mut currency = None;

    sc.skip_ws();
    if sc.peek()? == '}' {
        sc.next();
    } else {
        loop {
            sc.skip_ws();
            let key = sc.parse_string()?;
            sc.skip_ws();
            if sc.next()? != ':' {
                return None;
            }
            sc.skip_ws();
            let value = sc.parse_value()?;
            match key.as_str() {
                "amount_minor" => amount_minor = Some(value),
                "currency" => currency = Some(value),
                _ => {}
            }
            sc.skip_ws();
            match sc.next()? {
                ',' => continue,
                '}' => break,
                _ => return None,
            }
        }
    }

    sc.skip_ws();
    if sc.pos != sc.chars.len() {
        return None; // trailing garbage after the object
    }
    Some(Fields {
        amount_minor,
        currency,
    })
}

/// Character cursor over the wire string.
struct Scanner {
    chars: Vec<char>,
    pos: usize,
}

impl Scanner {
    fn new(s: &str) -> Scanner {
        Scanner {
            chars: s.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn next(&mut self) -> Option<char> {
        let c = self.chars.get(self.pos).copied()?;
        self.pos += 1;
        Some(c)
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    /// Parse a JSON string starting at the opening quote.
    fn parse_string(&mut self) -> Option<String> {
        if self.next()? != '"' {
            return None;
        }
        let mut out = String::new();
        loop {
            match self.next()? {
                '"' => return Some(out),
                '\\' => match self.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'n' => out.push('\n'),
                    't' => out.push('\t'),
                    'r' => out.push('\r'),
                    'b' => out.push('\u{0008}'),
                    'f' => out.push('\u{000C}'),
                    'u' => {
                        let mut code = 0u32;
                        for _ in 0..4 {
                            code = code * 16 + self.next()?.to_digit(16)?;
                        }
                        out.push(char::from_u32(code)?);
                    }
                    _ => return None,
                },
                c => out.push(c),
            }
        }
    }

    /// Parse a value: either a JSON string or an opaque non-string token (number,
    /// bool, null) consumed up to the next `,`, `}`, or whitespace.
    fn parse_value(&mut self) -> Option<WireValue> {
        match self.peek()? {
            '"' => Some(WireValue::Str(self.parse_string()?)),
            _ => {
                let start = self.pos;
                while let Some(c) = self.peek() {
                    if c == ',' || c == '}' || c.is_whitespace() {
                        break;
                    }
                    self.pos += 1;
                }
                if self.pos == start {
                    None
                } else {
                    Some(WireValue::NonStr)
                }
            }
        }
    }
}
