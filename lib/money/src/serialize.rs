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
use crate::Currency;

impl Money {
    /// Serialize to the canonical v1 wire format (TS001 §2.10). Exact; never lossy.
    /// (AC-S-1, AC-S-2.)
    pub fn serialize(&self) -> String {
        format!(
            "{{ \"amount_minor\": \"{}\", \"currency\": \"{}\" }}",
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
        let mut parser = WireParser::new(wire);
        let fields = parser.parse_object()?;

        let amount = match fields
            .amount_minor
            .ok_or(DeserializeError::MalformedWireValue)?
        {
            WireValue::String(value) => parse_amount_minor(&value)?,
            WireValue::Number => return Err(DeserializeError::MalformedWireValue),
        };
        let currency = match fields
            .currency
            .ok_or(DeserializeError::MalformedWireValue)?
        {
            WireValue::String(value) => {
                Currency::from_iso_code(&value).ok_or(DeserializeError::UnknownCurrency)?
            }
            WireValue::Number => return Err(DeserializeError::MalformedWireValue),
        };

        Ok(Money::new(amount, currency))
    }
}

#[derive(Debug, Default)]
struct WireFields {
    amount_minor: Option<WireValue>,
    currency: Option<WireValue>,
}

#[derive(Debug)]
enum WireValue {
    String(String),
    Number,
}

struct WireParser<'a> {
    input: &'a str,
    index: usize,
}

impl<'a> WireParser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, index: 0 }
    }

    fn parse_object(&mut self) -> Result<WireFields, DeserializeError> {
        let mut fields = WireFields::default();
        self.skip_ws();
        self.expect_byte(b'{')?;
        self.skip_ws();
        if self.consume_byte(b'}') {
            self.skip_ws();
            return if self.is_done() {
                Ok(fields)
            } else {
                Err(DeserializeError::MalformedWireValue)
            };
        }

        loop {
            let key = self.parse_string()?;
            self.skip_ws();
            self.expect_byte(b':')?;
            self.skip_ws();
            let value = self.parse_value()?;

            match key.as_str() {
                "amount_minor" => {
                    if fields.amount_minor.replace(value).is_some() {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                }
                "currency" => {
                    if fields.currency.replace(value).is_some() {
                        return Err(DeserializeError::MalformedWireValue);
                    }
                }
                _ => return Err(DeserializeError::MalformedWireValue),
            }

            self.skip_ws();
            if self.consume_byte(b'}') {
                self.skip_ws();
                return if self.is_done() {
                    Ok(fields)
                } else {
                    Err(DeserializeError::MalformedWireValue)
                };
            }
            self.expect_byte(b',')?;
            self.skip_ws();
        }
    }

    fn parse_value(&mut self) -> Result<WireValue, DeserializeError> {
        match self.peek_byte() {
            Some(b'"') => self.parse_string().map(WireValue::String),
            Some(b'-' | b'0'..=b'9') => {
                self.consume_number()?;
                Ok(WireValue::Number)
            }
            _ => Err(DeserializeError::MalformedWireValue),
        }
    }

    fn parse_string(&mut self) -> Result<String, DeserializeError> {
        self.expect_byte(b'"')?;
        let mut value = String::new();
        while let Some(byte) = self.peek_byte() {
            self.index += 1;
            match byte {
                b'"' => return Ok(value),
                b'\\' => return Err(DeserializeError::MalformedWireValue),
                0x00..=0x1f => return Err(DeserializeError::MalformedWireValue),
                _ => value.push(char::from(byte)),
            }
        }
        Err(DeserializeError::MalformedWireValue)
    }

    fn consume_number(&mut self) -> Result<(), DeserializeError> {
        if self.consume_byte(b'-') && !self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            return Err(DeserializeError::MalformedWireValue);
        }

        let mut digits = 0;
        while self.peek_byte().is_some_and(|b| b.is_ascii_digit()) {
            self.index += 1;
            digits += 1;
        }
        if digits == 0 {
            return Err(DeserializeError::MalformedWireValue);
        }

        if self
            .peek_byte()
            .is_some_and(|b| matches!(b, b'.' | b'e' | b'E'))
        {
            while self.peek_byte().is_some_and(|b| {
                b.is_ascii_digit() || matches!(b, b'.' | b'e' | b'E' | b'+' | b'-')
            }) {
                self.index += 1;
            }
        }

        Ok(())
    }

    fn skip_ws(&mut self) {
        while self
            .peek_byte()
            .is_some_and(|b| matches!(b, b' ' | b'\n' | b'\r' | b'\t'))
        {
            self.index += 1;
        }
    }

    fn expect_byte(&mut self, byte: u8) -> Result<(), DeserializeError> {
        if self.consume_byte(byte) {
            Ok(())
        } else {
            Err(DeserializeError::MalformedWireValue)
        }
    }

    fn consume_byte(&mut self, byte: u8) -> bool {
        if self.peek_byte() == Some(byte) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.as_bytes().get(self.index).copied()
    }

    fn is_done(&self) -> bool {
        self.index == self.input.len()
    }
}

fn parse_amount_minor(value: &str) -> Result<i64, DeserializeError> {
    if !is_canonical_integer(value) {
        return Err(DeserializeError::InvalidAmountMinor);
    }
    value
        .parse::<i64>()
        .map_err(|_| DeserializeError::AmountOutOfRange)
}

fn is_canonical_integer(value: &str) -> bool {
    let Some(rest) = value.strip_prefix('-') else {
        return is_unsigned_canonical_integer(value);
    };
    !rest.is_empty() && rest != "0" && is_unsigned_canonical_integer(rest)
}

fn is_unsigned_canonical_integer(value: &str) -> bool {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return false;
    }
    value == "0" || !value.starts_with('0')
}
