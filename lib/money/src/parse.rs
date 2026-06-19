//! Parsing free-form amount strings into `Money` (TS001 §2.4, §2.5).

use crate::currency::Currency;
use crate::error::ParseError;
use crate::money::Money;

/// Caller-selected rounding modes (TS001 §2.5).
///
/// The type is defined in Phase 1 because [`ParseOptions`] references it, but
/// rounding *behaviour* (rounding-enabled parsing, scaling) is Phase 2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RoundingMode {
    /// Ties away from zero.
    HalfUp,
    /// Ties to the nearest even minor unit (banker's rounding).
    HalfEven,
    /// Toward zero (truncate).
    Down,
    /// Away from zero.
    Up,
    /// Toward positive infinity.
    Ceiling,
    /// Toward negative infinity.
    Floor,
}

/// Options controlling [`Money::parse`]. Forward-extensible (TS001 §2.9).
///
/// `rounding == None` (the default) means reject inputs with more fractional
/// digits than the currency's exponent ([`ParseError::TooManyFractionalDigits`]),
/// so scraped data is never silently corrupted (TS001 §2.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ParseOptions {
    /// Rounding mode to apply to excess fractional digits, or `None` to reject
    /// them. Rounding behaviour is Phase 2.
    pub rounding: Option<RoundingMode>,
}

impl Money {
    /// Parse a free-form amount string given the expected `currency`.
    ///
    /// Implements the TS001 §2.4 parsing algorithm: length guard, version-stable
    /// fold map, sign and currency-indicator extraction, group/decimal-separator
    /// disambiguation, fraction handling driven by `currency.exponent()`, and
    /// exact i64 assembly. Returns a typed [`ParseError`] on any malformed input.
    ///
    /// Phase 1 delivers positive baseline parsing (AC-P baseline rows, AC-P-AMB,
    /// AC-P-NEG, AC-P-ZERO-3/4); negative amounts and rounding-enabled parsing are
    /// Phase 2.
    pub fn parse(
        raw: &str,
        currency: Currency,
        options: ParseOptions,
    ) -> Result<Money, ParseError> {
        if raw.chars().count() > 256 {
            return Err(ParseError::InputTooLong);
        }
        if options.rounding.is_some() {
            // Rounding behavior is Phase 2; Phase 1 keeps reject-by-default semantics.
        }

        let mut normalized = normalize(raw);
        normalized = normalized.trim_matches(' ').to_string();
        if normalized.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        let (payload, is_negative) = extract_phase1_sign(&normalized)?;
        let payload = extract_currency_indicators(&payload, currency)?;
        let payload = validate_and_remove_group_spaces(&payload)?;
        validate_number_characters(&payload)?;

        let ParsedNumber {
            integer_digits,
            fraction_digits,
        } = parse_number_parts(&payload)?;

        let exponent = usize::from(currency.exponent());
        if fraction_digits.len() > exponent {
            return Err(ParseError::TooManyFractionalDigits);
        }

        let mut amount_digits = integer_digits;
        amount_digits.push_str(&fraction_digits);
        for _ in fraction_digits.len()..exponent {
            amount_digits.push('0');
        }

        let amount_minor = amount_digits
            .parse::<i64>()
            .map_err(|_| ParseError::Overflow)?;

        if is_negative {
            return Err(ParseError::MalformedSign);
        }

        Ok(Money::new(amount_minor, currency))
    }
}

struct ParsedNumber {
    integer_digits: String,
    fraction_digits: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CurrencyToken {
    text: &'static str,
    currency: Option<Currency>,
    ambiguous_dollar: bool,
}

const CURRENCY_TOKENS: &[CurrencyToken] = &[
    CurrencyToken {
        text: "CAN$",
        currency: Some(Currency::CAD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "CAD$",
        currency: Some(Currency::CAD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "AUD$",
        currency: Some(Currency::AUD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "US$",
        currency: Some(Currency::USD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "USD",
        currency: Some(Currency::USD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "EUR",
        currency: Some(Currency::EUR),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "CAD",
        currency: Some(Currency::CAD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "AUD",
        currency: Some(Currency::AUD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "CA$",
        currency: Some(Currency::CAD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "AU$",
        currency: Some(Currency::AUD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "C$",
        currency: Some(Currency::CAD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "A$",
        currency: Some(Currency::AUD),
        ambiguous_dollar: false,
    },
    CurrencyToken {
        text: "$",
        currency: None,
        ambiguous_dollar: true,
    },
    CurrencyToken {
        text: "€",
        currency: Some(Currency::EUR),
        ambiguous_dollar: false,
    },
];

fn normalize(raw: &str) -> String {
    raw.chars()
        .map(|ch| match ch {
            '\u{ff10}'..='\u{ff19}' => {
                char::from_u32(u32::from(ch) - u32::from('\u{ff10}') + u32::from('0')).unwrap()
            }
            '\u{ff04}' => '$',
            '\u{ff0e}' => '.',
            '\u{ff0c}' => ',',
            '\u{2212}' => '-',
            '\u{00a0}' | '\u{2009}' | '\u{202f}' | '\u{2007}' => ' ',
            _ => ch,
        })
        .collect()
}

fn extract_phase1_sign(input: &str) -> Result<(String, bool), ParseError> {
    let minus_count = input.chars().filter(|&ch| ch == '-').count();
    let plus_count = input.chars().filter(|&ch| ch == '+').count();
    let opens = input.chars().filter(|&ch| ch == '(').count();
    let closes = input.chars().filter(|&ch| ch == ')').count();

    if opens > 0 || closes > 0 {
        if opens == 1 && closes == 1 && input.starts_with('(') && input.ends_with(')') {
            return Ok((
                input[1..input.len() - 1].trim_matches(' ').to_string(),
                true,
            ));
        }
        return Err(ParseError::MalformedSign);
    }

    if minus_count > 1 || plus_count > 1 || (minus_count == 1 && plus_count == 1) {
        return Err(ParseError::MalformedSign);
    }
    if minus_count == 1 {
        return Ok((input.replace('-', "").trim_matches(' ').to_string(), true));
    }
    if plus_count == 1 {
        if let Some(rest) = input.strip_prefix('+') {
            return Ok((rest.trim_matches(' ').to_string(), false));
        }
        return Err(ParseError::MalformedSign);
    }

    Ok((input.to_string(), false))
}

fn extract_currency_indicators(input: &str, expected: Currency) -> Result<String, ParseError> {
    let mut payload = input.trim_matches(' ').to_string();
    let mut leading = None;
    let mut trailing = None;

    if let Some((token, rest)) = consume_leading_token(&payload) {
        validate_token(token, expected)?;
        leading = Some(token);
        payload = rest.trim_matches(' ').to_string();
    }

    if let Some((token, rest)) = consume_trailing_token(&payload) {
        validate_token(token, expected)?;
        trailing = Some(token);
        payload = rest.trim_matches(' ').to_string();
        if consume_trailing_token(&payload).is_some() {
            return Err(ParseError::MalformedCurrency);
        }
    }

    if leading.is_some() && trailing.is_some() {
        return Err(ParseError::MalformedCurrency);
    }

    Ok(payload)
}

fn validate_token(token: CurrencyToken, expected: Currency) -> Result<(), ParseError> {
    if token.ambiguous_dollar {
        return match expected {
            Currency::USD | Currency::CAD | Currency::AUD => Ok(()),
            Currency::EUR => Err(ParseError::CurrencyMismatch),
        };
    }
    if token.currency == Some(expected) {
        Ok(())
    } else {
        Err(ParseError::CurrencyMismatch)
    }
}

fn consume_leading_token(input: &str) -> Option<(CurrencyToken, String)> {
    let trimmed = input.trim_start_matches(' ');
    for &token in CURRENCY_TOKENS {
        if starts_with_token(trimmed, token.text) {
            let rest = &trimmed[token.text.len()..];
            return Some((token, rest.to_string()));
        }
    }
    None
}

fn consume_trailing_token(input: &str) -> Option<(CurrencyToken, String)> {
    let trimmed = input.trim_end_matches(' ');
    for &token in CURRENCY_TOKENS {
        if ends_with_token(trimmed, token.text) {
            let rest_len = trimmed.len() - token.text.len();
            return Some((token, trimmed[..rest_len].to_string()));
        }
    }
    None
}

fn starts_with_token(input: &str, token: &str) -> bool {
    input
        .get(..token.len())
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case(token) || prefix == token)
}

fn ends_with_token(input: &str, token: &str) -> bool {
    input
        .get(input.len().saturating_sub(token.len())..)
        .is_some_and(|suffix| suffix.eq_ignore_ascii_case(token) || suffix == token)
}

fn validate_and_remove_group_spaces(input: &str) -> Result<String, ParseError> {
    if !input.contains(' ') {
        return Ok(input.to_string());
    }

    let groups: Vec<&str> = input.split(' ').filter(|group| !group.is_empty()).collect();
    if groups.len() < 2 {
        return Ok(input.replace(' ', ""));
    }
    if !is_all_digits(groups[0]) || groups[0].is_empty() || groups[0].len() > 3 {
        return Err(ParseError::InvalidGrouping);
    }
    for group in &groups[1..groups.len() - 1] {
        if group.len() != 3 || !is_all_digits(group) {
            return Err(ParseError::InvalidGrouping);
        }
    }
    let last = groups[groups.len() - 1];
    let comma = last.find(',');
    let dot = last.find('.');
    let first_separator = match (comma, dot) {
        (Some(comma), Some(dot)) => Some(comma.min(dot)),
        (Some(comma), None) => Some(comma),
        (None, Some(dot)) => Some(dot),
        (None, None) => None,
    };
    let integer_part = first_separator.map_or(last, |index| &last[..index]);
    if integer_part.len() != 3 || !is_all_digits(integer_part) {
        return Err(ParseError::InvalidGrouping);
    }

    Ok(input.replace(' ', ""))
}

fn validate_number_characters(input: &str) -> Result<(), ParseError> {
    if !input
        .chars()
        .all(|ch| ch.is_ascii_digit() || ch == '.' || ch == ',')
    {
        return Err(ParseError::InvalidCharacter);
    }
    if !input.chars().any(|ch| ch.is_ascii_digit()) {
        return Err(ParseError::MalformedNumber);
    }
    Ok(())
}

fn parse_number_parts(input: &str) -> Result<ParsedNumber, ParseError> {
    let dot_count = input.chars().filter(|&ch| ch == '.').count();
    let comma_count = input.chars().filter(|&ch| ch == ',').count();
    let (integer_part, fraction_digits, group_separator) = match (dot_count, comma_count) {
        (0, 0) => (input, "", None),
        (dots, commas) if dots > 0 && commas > 0 => split_mixed_separators(input)?,
        (_, _) => split_single_separator(input)?,
    };

    let integer_digits = validate_integer_groups(integer_part, group_separator)?;
    Ok(ParsedNumber {
        integer_digits,
        fraction_digits: fraction_digits.to_string(),
    })
}

fn split_mixed_separators(input: &str) -> Result<(&str, &str, Option<char>), ParseError> {
    let decimal_index = input.rfind([',', '.']).ok_or(ParseError::MalformedNumber)?;
    let decimal = input[decimal_index..].chars().next().unwrap();
    let group = if decimal == '.' { ',' } else { '.' };
    let integer = &input[..decimal_index];
    let fraction = &input[decimal_index + decimal.len_utf8()..];

    if fraction.is_empty() {
        return Err(ParseError::MalformedNumber);
    }
    if integer.contains(decimal) {
        return Err(ParseError::InvalidGrouping);
    }
    if fraction.contains('.') || fraction.contains(',') {
        return Err(ParseError::InvalidGrouping);
    }
    if integer.contains(group) {
        Ok((integer, fraction, Some(group)))
    } else {
        Ok((integer, fraction, None))
    }
}

fn split_single_separator(input: &str) -> Result<(&str, &str, Option<char>), ParseError> {
    let separator = if input.contains('.') { '.' } else { ',' };
    let count = input.chars().filter(|&ch| ch == separator).count();
    if count > 1 {
        return Ok((input, "", Some(separator)));
    }

    let index = input.rfind(separator).unwrap();
    let digits_after = input[index + 1..].len();
    match digits_after {
        0 => Err(ParseError::MalformedNumber),
        1 | 2 => Ok((&input[..index], &input[index + 1..], None)),
        3 => Ok((input, "", Some(separator))),
        _ => Err(ParseError::InvalidGrouping),
    }
}

fn validate_integer_groups(
    input: &str,
    group_separator: Option<char>,
) -> Result<String, ParseError> {
    if input.is_empty() {
        return Err(ParseError::MalformedNumber);
    }

    let digits = if let Some(separator) = group_separator {
        let groups: Vec<&str> = input.split(separator).collect();
        if groups.is_empty()
            || groups[0].is_empty()
            || groups[0].len() > 3
            || !is_all_digits(groups[0])
        {
            return Err(ParseError::InvalidGrouping);
        }
        for group in &groups[1..] {
            if group.len() != 3 || !is_all_digits(group) {
                return Err(ParseError::InvalidGrouping);
            }
        }
        groups.join("")
    } else {
        if !is_all_digits(input) {
            return Err(ParseError::MalformedNumber);
        }
        input.to_string()
    };

    if digits.len() > 1 && digits.starts_with('0') {
        return Err(ParseError::InvalidGrouping);
    }

    Ok(digits)
}

fn is_all_digits(input: &str) -> bool {
    input.chars().all(|ch| ch.is_ascii_digit())
}
