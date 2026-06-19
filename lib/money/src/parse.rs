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
        // Phase 1 ignores `options.rounding`: rounding *behaviour* is Phase 2, so
        // excess fractional digits are always rejected (step 8). The field is read
        // here only to document that Phase 1 deliberately does not act on it.
        let _ = options.rounding;

        // Step 0a — length guard on the RAW input (before normalization), a cheap
        // denial-of-service bound (TS001 §2.4 step 0; AC-P-NEG-21).
        if raw.chars().count() > MAX_INPUT_CHARS {
            return Err(ParseError::InputTooLong);
        }

        // Step 0b — version-stable fold map (explicit, runtime-independent). Phase 1
        // folds every §2.4 step-0 entry that affects the positive parse path: the
        // five §2.3 spaces to ASCII space (so step 4 sees one space character
        // regardless of source), full-width digits to ASCII digits, and the
        // full-width `＄`/`．`/`，` to their ASCII forms (AC-P-26).
        //
        // The U+2212 minus fold is deliberately deferred: it is purely a sign
        // character with no positive-path effect, so it ships with §2.4 step-2 sign
        // extraction in Phase 2. Until then a U+2212 survives to fail the step-5
        // whitelist as `InvalidCharacter`, exactly like a leading ASCII `-`.
        let folded: String = raw
            .chars()
            .map(|c| match c {
                '\u{00A0}' | '\u{2009}' | '\u{202F}' | '\u{2007}' => ' ',
                '\u{FF10}'..='\u{FF19}' => {
                    // Full-width digit → ASCII digit (offset by the block distance).
                    char::from_u32(c as u32 - 0xFF10 + 0x30).unwrap()
                }
                '\u{FF04}' => '$',
                '\u{FF0E}' => '.',
                '\u{FF0C}' => ',',
                other => other,
            })
            .collect();

        // Step 1 — trim; empty input is rejected.
        let trimmed = folded.trim();
        if trimmed.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Step 2 — sign extraction is Phase 2. With no handling here, a leading or
        // trailing `-`, `+`, or accounting parentheses survive to the step-5
        // whitelist and become `InvalidCharacter` (an error, never a panic).

        // Step 3 — currency-indicator extraction (longest match, at most one leading
        // and one trailing token), ignoring spaces between indicator and number.
        let mut payload = trimmed;
        let mut leading: Option<&str> = None;
        let mut trailing: Option<&str> = None;

        if let Some((token, token_currency)) = match_leading_indicator(payload) {
            check_indicator(token_currency, currency)?;
            leading = Some(token);
            payload = payload[token.len()..].trim_start();
        }
        if let Some((token, token_currency)) = match_trailing_indicator(payload) {
            check_indicator(token_currency, currency)?;
            trailing = Some(token);
            payload = payload[..payload.len() - token.len()].trim_end();
        }
        // Two distinct indicator tokens (e.g. `$…USD`, `CAD$…CAD`) are contradictory.
        if let (Some(l), Some(t)) = (leading, trailing) {
            if !l.eq_ignore_ascii_case(t) {
                return Err(ParseError::MalformedCurrency);
            }
        }
        // A second indicator on the same side (e.g. `1.234,56 € EUR`, where one
        // trailing token is consumed but another remains) is also multiple distinct
        // tokens → MalformedCurrency (TS001 §2.4 step 3; AC-P-NEG-14). A bare number
        // never matches an indicator, so legitimate amounts are unaffected.
        if match_leading_indicator(payload).is_some() || match_trailing_indicator(payload).is_some()
        {
            return Err(ParseError::MalformedCurrency);
        }

        // Step 4 — validate and remove group spacing. Listed spaces act only as group
        // separators: the first group is 1–3 digits and every later group exactly 3,
        // with any decimal separator and fraction staying attached to the final group.
        let despaced: String = if payload.contains(' ') {
            validate_space_groups(payload)?;
            payload.replace(' ', "")
        } else {
            payload.to_string()
        };

        // Step 5 — character whitelist and digit presence.
        if !despaced
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == ',')
        {
            return Err(ParseError::InvalidCharacter);
        }
        if !despaced.chars().any(|c| c.is_ascii_digit()) {
            return Err(ParseError::MalformedNumber);
        }

        // Steps 6–7 — decimal-separator disambiguation and integer/group validation.
        let (integer_digits, fraction) = split_number(&despaced)?;

        // Step 8 — fraction handling against the currency exponent.
        let exponent = currency.exponent() as usize;
        if fraction.len() > exponent {
            return Err(ParseError::TooManyFractionalDigits);
        }
        let mut padded_fraction = fraction.to_string();
        while padded_fraction.len() < exponent {
            padded_fraction.push('0');
        }

        // Step 9 — assemble minor units exactly; reject values outside i64.
        let digits = format!("{integer_digits}{padded_fraction}");
        let amount_minor = digits.parse::<i64>().map_err(|_| ParseError::Overflow)?;

        // Step 10 — done. (Phase 1 is positive-only, so no sign is applied.)
        Ok(Money::new(amount_minor, currency))
    }
}

/// Maximum accepted input length in Unicode scalar values (TS001 §2.4 step 0).
const MAX_INPUT_CHARS: usize = 256;

/// Accepted currency indicators paired with the currency they denote, `None`
/// meaning the ambiguous bare `$` (valid for USD/CAD/AUD). Ordered longest-token
/// first so prefix/suffix matching is longest-match (TS001 §2.2, §2.4 step 3).
const INDICATORS: &[(&str, Option<Currency>)] = &[
    ("Can$", Some(Currency::CAD)),
    ("CAD$", Some(Currency::CAD)),
    ("AUD$", Some(Currency::AUD)),
    ("CAD", Some(Currency::CAD)),
    ("AUD", Some(Currency::AUD)),
    ("USD", Some(Currency::USD)),
    ("EUR", Some(Currency::EUR)),
    ("US$", Some(Currency::USD)),
    ("CA$", Some(Currency::CAD)),
    ("AU$", Some(Currency::AUD)),
    ("€", Some(Currency::EUR)),
    ("C$", Some(Currency::CAD)),
    ("A$", Some(Currency::AUD)),
    ("$", None),
];

/// Longest-match the leading currency indicator (case-insensitive).
fn match_leading_indicator(s: &str) -> Option<(&'static str, Option<Currency>)> {
    for &(token, token_currency) in INDICATORS {
        if let Some(head) = s.get(..token.len()) {
            if head.eq_ignore_ascii_case(token) {
                return Some((token, token_currency));
            }
        }
    }
    None
}

/// Longest-match the trailing currency indicator (case-insensitive).
fn match_trailing_indicator(s: &str) -> Option<(&'static str, Option<Currency>)> {
    for &(token, token_currency) in INDICATORS {
        if s.len() >= token.len() {
            if let Some(tail) = s.get(s.len() - token.len()..) {
                if tail.eq_ignore_ascii_case(token) {
                    return Some((token, token_currency));
                }
            }
        }
    }
    None
}

/// Validate that an indicator token is consistent with the expected currency.
/// A bare `$` is accepted for USD/CAD/AUD; for EUR it is a mismatch (§2.4 step 3).
fn check_indicator(token_currency: Option<Currency>, expected: Currency) -> Result<(), ParseError> {
    match token_currency {
        None => {
            if matches!(expected, Currency::USD | Currency::CAD | Currency::AUD) {
                Ok(())
            } else {
                Err(ParseError::CurrencyMismatch)
            }
        }
        Some(c) if c == expected => Ok(()),
        Some(_) => Err(ParseError::CurrencyMismatch),
    }
}

/// Step 4 grouping rule for space-separated groups: first group 1–3 digits, every
/// later group exactly 3 leading digits (a decimal separator + fraction may trail
/// the final group). Returns `InvalidGrouping` on violation.
fn validate_space_groups(s: &str) -> Result<(), ParseError> {
    let groups: Vec<&str> = s.split(' ').collect();
    let last_index = groups.len() - 1;
    for (i, group) in groups.iter().enumerate() {
        let digit_run = group.chars().take_while(|c| c.is_ascii_digit()).count();
        // Only the final group may carry the decimal separator and fraction; every
        // other group must be digits all the way through.
        if i != last_index && digit_run != group.len() {
            return Err(ParseError::InvalidGrouping);
        }
        let ok = if i == 0 {
            (1..=3).contains(&digit_run)
        } else {
            digit_run == 3
        };
        if !ok {
            return Err(ParseError::InvalidGrouping);
        }
    }
    Ok(())
}

/// Steps 6–7: disambiguate the decimal separator and validate integer grouping,
/// returning `(integer_digits, fraction_digits)` with all group separators removed.
/// `s` contains only ASCII digits, `.`, and `,`.
fn split_number(s: &str) -> Result<(String, String), ParseError> {
    let has_dot = s.contains('.');
    let has_comma = s.contains(',');

    // (integer_part_with_group_seps, group_sep, fraction_digits)
    let (integer_part, group_sep, fraction): (&str, Option<char>, &str) = match (has_dot, has_comma)
    {
        (true, true) => {
            // Right-most separator is the decimal; the other is the group sep.
            let decimal_sep = if s.rfind('.') > s.rfind(',') {
                '.'
            } else {
                ','
            };
            let group_sep = if decimal_sep == '.' { ',' } else { '.' };
            let pos = s.rfind(decimal_sep).unwrap();
            let integer_part = &s[..pos];
            let fraction = &s[pos + 1..];
            // Exactly one decimal separator, and the fraction must be plain digits.
            if integer_part.contains(decimal_sep)
                || fraction.contains('.')
                || fraction.contains(',')
            {
                return Err(ParseError::InvalidGrouping);
            }
            (integer_part, Some(group_sep), fraction)
        }
        (true, false) | (false, true) => {
            let sep = if has_dot { '.' } else { ',' };
            let count = s.matches(sep).count();
            let last = s.rfind(sep).unwrap();
            let after = s.len() - (last + 1);
            if count > 1 {
                // Repeated single separator → grouping, no fractional part.
                (s, Some(sep), "")
            } else if after == 3 {
                // Lone separator before exactly three digits → grouping default.
                (s, Some(sep), "")
            } else if after == 1 || after == 2 {
                (&s[..last], None, &s[last + 1..])
            } else if after == 0 {
                return Err(ParseError::MalformedNumber);
            } else {
                return Err(ParseError::InvalidGrouping);
            }
        }
        (false, false) => (s, None, ""),
    };

    // Step 7 — group/integer validation, then strip group separators.
    let integer_digits = if let Some(sep) = group_sep {
        let groups: Vec<&str> = integer_part.split(sep).collect();
        for (i, group) in groups.iter().enumerate() {
            if !group.chars().all(|c| c.is_ascii_digit()) {
                return Err(ParseError::InvalidGrouping);
            }
            let ok = if i == 0 {
                (1..=3).contains(&group.len())
            } else {
                group.len() == 3
            };
            if !ok {
                return Err(ParseError::InvalidGrouping);
            }
        }
        groups.concat()
    } else {
        if !integer_part.chars().all(|c| c.is_ascii_digit()) {
            return Err(ParseError::InvalidGrouping);
        }
        integer_part.to_string()
    };

    // A leading zero is allowed only when the integer part is exactly `0`.
    if integer_digits.len() > 1 && integer_digits.starts_with('0') {
        return Err(ParseError::InvalidGrouping);
    }

    Ok((integer_digits, fraction.to_string()))
}
