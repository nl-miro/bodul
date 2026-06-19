//! Parsing free-form amount strings into `Money` (TS001 §2.4, §2.5).

use crate::currency::{Currency, ALL_INDICATORS};
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

// ---------------------------------------------------------------------------
// Step 0: fold map — version-stable pre-normalization (TS001 §2.4 step 0)
// ---------------------------------------------------------------------------

fn fold_char(c: char) -> char {
    match c {
        '\u{FF10}'..='\u{FF19}' => char::from_u32(c as u32 - 0xFF10 + 0x0030).unwrap(),
        '\u{FF04}' => '$',
        '\u{FF0E}' => '.',
        '\u{FF0C}' => ',',
        '\u{2212}' => '-',
        '\u{00A0}' | '\u{2009}' | '\u{202F}' | '\u{2007}' => ' ',
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Helpers for sign extraction (step 2)
// ---------------------------------------------------------------------------

/// Check whether `s` ends with `suffix`, case-insensitive for ASCII suffixes,
/// exact for non-ASCII (e.g. `€`).
fn ends_with_indicator(s: &str, suffix: &str) -> bool {
    if s.len() < suffix.len() {
        return false;
    }
    let start = s.len() - suffix.len();
    // Guard: the slice boundary must fall on a UTF-8 char boundary.
    if !s.is_char_boundary(start) {
        return false;
    }
    let candidate = &s[start..];
    if suffix.chars().all(|c| c.is_ascii()) {
        candidate.eq_ignore_ascii_case(suffix)
    } else {
        candidate == suffix
    }
}

/// Check whether `s` starts with `prefix`, case-insensitive for ASCII prefixes,
/// exact for non-ASCII.
fn starts_with_indicator(s: &str, prefix: &str) -> bool {
    if s.len() < prefix.len() {
        return false;
    }
    // Guard: the slice boundary must fall on a UTF-8 char boundary.
    if !s.is_char_boundary(prefix.len()) {
        return false;
    }
    let candidate = &s[..prefix.len()];
    if prefix.chars().all(|c| c.is_ascii()) {
        candidate.eq_ignore_ascii_case(prefix)
    } else {
        candidate == prefix
    }
}

/// Detect a `-` immediately preceding a trailing currency indicator with zero or
/// more spaces between them (TS001 §2.4 step 2). Returns the byte position of
/// the `-` if found, and the length of the matched indicator.
fn find_dash_before_trailing_indicator(s: &str) -> Option<(usize, usize)> {
    for (indicator, _) in ALL_INDICATORS {
        if ends_with_indicator(s, indicator) {
            let before = &s[..s.len() - indicator.len()];
            if let Some(dash_pos) = before.rfind('-') {
                let between = &before[dash_pos + 1..];
                if between.chars().all(|c| c == ' ') {
                    return Some((dash_pos, indicator.len()));
                }
            }
            break; // longest match wins
        }
    }
    None
}

/// Extract sign markers from `s`, modifying `s` in place by removing the sign
/// delimiters. Returns `true` if the amount is negative.
/// Also detects at-most-one sign; conflicting/duplicate markers → `MalformedSign`.
fn extract_sign(s: &mut String) -> Result<bool, ParseError> {
    let mut negative = false;
    let mut sign_found = false;

    // Accounting parentheses
    if s.starts_with('(') && s.ends_with(')') {
        let inner = &s[1..s.len() - 1];
        let inner = inner.trim();
        if inner.is_empty() {
            return Err(ParseError::MalformedSign);
        }
        *s = inner.to_string();
        return Ok(true); // negative, done
    }

    // Leading +/-
    if let Some(rest) = s.strip_prefix('-') {
        negative = true;
        sign_found = true;
        *s = rest.trim_start().to_string();
    } else if let Some(rest) = s.strip_prefix('+') {
        sign_found = true;
        *s = rest.trim_start().to_string();
    }

    // Trailing minus (last non-space char is '-')
    if !sign_found && !s.is_empty() {
        let trimmed = s.trim_end();
        if let Some(rest) = trimmed.strip_suffix('-') {
            negative = true;
            sign_found = true;
            *s = rest.trim_end().to_string();
        }
    }

    // Minus immediately preceding a trailing currency indicator
    if !sign_found {
        if let Some((dash_pos, indicator_len)) = find_dash_before_trailing_indicator(s) {
            let indicator_start = s.len() - indicator_len;
            let mut new_s = String::with_capacity(s.len());
            new_s.push_str(&s[..dash_pos]);
            new_s.push_str(&s[indicator_start..]);
            *s = new_s.trim().to_string();
            negative = true;
            sign_found = true;
        }
    }

    // Check for conflicting signs: after extracting one sign, we shouldn't find another
    if sign_found {
        // Check that no additional sign markers remain
        if s.starts_with('-')
            || s.starts_with('+')
            || (s.starts_with('(') && s.ends_with(')'))
        {
            return Err(ParseError::MalformedSign);
        }
        // Check trailing minus
        if let Some(trimmed) = s.strip_suffix('-') {
            if trimmed.len() < s.len() {
                return Err(ParseError::MalformedSign);
            }
        }
        // Check - before trailing indicator again
        if find_dash_before_trailing_indicator(s).is_some() {
            return Err(ParseError::MalformedSign);
        }
    }

    Ok(negative)
}

// ---------------------------------------------------------------------------
// Helpers for currency-indicator extraction (step 3)
// ---------------------------------------------------------------------------

/// Result of matching a currency indicator at either end of the string.
struct IndicatorMatch {
    /// The raw text that was matched (for detecting distinct tokens).
    _text: String,
    /// The currency the indicator belongs to.
    currency: Currency,
    /// Whether the indicator is ambiguous (bare `$`).
    ambiguous: bool,
    /// How many bytes the indicator occupied from the start/end.
    len: usize,
}

/// Try to extract a leading currency indicator from `s`. Returns `None` if no
/// indicator is found at the front (ignoring a single space between indicator
/// and number).
fn match_leading_indicator(s: &str) -> Option<IndicatorMatch> {
    for (indicator, currency) in ALL_INDICATORS {
        if starts_with_indicator(s, indicator) {
            let rest = &s[indicator.len()..];
            // Allow a single space between indicator and number (strip it)
            let rest = rest.strip_prefix(' ').unwrap_or(rest);
            let len = s.len() - rest.len();
            let ambiguous = *indicator == "$";
            return Some(IndicatorMatch {
                _text: indicator.to_string(),
                currency: *currency,
                ambiguous,
                len,
            });
        }
    }
    None
}

/// Try to extract a trailing currency indicator from `s`. Returns `None` if no
/// indicator is found at the back (ignoring a single space between number and
/// indicator).
fn match_trailing_indicator(s: &str) -> Option<IndicatorMatch> {
    for (indicator, currency) in ALL_INDICATORS {
        if ends_with_indicator(s, indicator) {
            let before = &s[..s.len() - indicator.len()];
            // Allow a single space between number and indicator (strip it)
            let before = before.strip_suffix(' ').unwrap_or(before);
            let len = s.len() - before.len();
            let ambiguous = *indicator == "$";
            return Some(IndicatorMatch {
                _text: indicator.to_string(),
                currency: *currency,
                ambiguous,
                len,
            });
        }
    }
    None
}

/// Extract currency indicators from `s`, modifying it in place. Validates that
/// indicators match the expected `currency` and that at most one distinct
/// indicator token is consumed per side.
fn extract_currency_indicators(
    s: &mut String,
    currency: Currency,
) -> Result<(), ParseError> {
    let leading = match_leading_indicator(s);
    let trailing = match_trailing_indicator(s);

    match (leading.as_ref(), trailing.as_ref()) {
        (None, None) => {
            // No currency indicators — fine
        }
        (Some(lead), Some(trail)) => {
            // Both leading and trailing indicators present
            let same_token = lead._text.to_lowercase() == trail._text.to_lowercase();

            if !same_token {
                return Err(ParseError::MalformedCurrency);
            }

            // Same indicator on both sides (e.g. "$5.00 $")
            // Remove trailing first, then re-match leading (may include space)
            let trail_start = s.len() - trail.len;
            s.replace_range(trail_start.., "");
            *s = s.trim().to_string();

            // Re-match the leading indicator after trailing removal
            let lead2 = match_leading_indicator(s);
            match lead2 {
                Some(lead2) if lead2._text.to_lowercase() == lead._text.to_lowercase() => {
                    // Remove the leading indicator (with any consumed space)
                    s.replace_range(..lead2.len, "");
                    *s = s.trim().to_string();
                    validate_indicator(&lead2, currency)?;
                }
                Some(_) => {
                    // Leading indicator changed after trailing removal — should not happen
                    return Err(ParseError::MalformedCurrency);
                }
                None => {
                    // Leading indicator was already consumed with trailing removal
                    // (e.g., input was just "$")
                    validate_indicator(lead, currency)?;
                }
            }
        }
        (Some(lead), None) => {
            validate_indicator(lead, currency)?;
            s.replace_range(..lead.len, "");
            *s = s.trim_start().to_string();

            // Check for an adjacent trailing indicator (two indicators at same end)
            if !s.is_empty() {
                if match_trailing_indicator(s).is_some() {
                    return Err(ParseError::MalformedCurrency);
                }
            }
        }
        (None, Some(trail)) => {
            validate_indicator(trail, currency)?;
            let trail_start = s.len() - trail.len;
            s.replace_range(trail_start.., "");
            *s = s.trim_end().to_string();

            // Check for an adjacent trailing indicator (two at same end)
            if !s.is_empty() {
                if match_trailing_indicator(s).is_some() {
                    return Err(ParseError::MalformedCurrency);
                }
            }
        }
    }

    Ok(())
}

fn validate_indicator(
    matched: &IndicatorMatch,
    expected: Currency,
) -> Result<(), ParseError> {
    if matched.ambiguous {
        // Bare `$` — must be USD, CAD, or AUD
        if expected == Currency::EUR {
            return Err(ParseError::CurrencyMismatch);
        }
        // Accepted for USD, CAD, AUD
        Ok(())
    } else {
        // Unambiguous indicator — must match expected currency exactly
        if matched.currency != expected {
            return Err(ParseError::CurrencyMismatch);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Helpers for space-group validation (step 4)
// ---------------------------------------------------------------------------

/// Validate space-delimited digit groups and remove spaces.
/// TS001 §2.4 step 4: listed spaces act only as group separators.
/// Groups must be: first 1–3 digits, every later exactly 3 digits.
fn remove_space_groups(s: &mut String) -> Result<(), ParseError> {
    // If no spaces, nothing to do
    if !s.contains(' ') {
        return Ok(());
    }

    // Split the string by ' ', filter out empty parts
    let parts: Vec<&str> = s.split(' ').filter(|p| !p.is_empty()).collect();

    if parts.is_empty() {
        return Ok(());
    }

    // If only one part, spaces were spurious (e.g. "1 234" without digits after space?)
    // Actually, spaces could be between number and stripped indicator.
    // If there's only one part, no spaces remain — just return
    if parts.len() == 1 {
        *s = parts[0].to_string();
        return Ok(());
    }

    // The last part may contain a decimal separator and fraction digits.
    // All parts except possibly the last are integer groups.
    let last = parts[parts.len() - 1];
    let decimal_sep = last.find('.').or_else(|| last.find(','));

    // Integer group digits (everything before the last part, plus the integer
    // portion of the last part)
    let mut integer_parts: Vec<&str> = parts[..parts.len() - 1]
        .iter()
        .copied()
        .collect();

    let last_int_part = if let Some(sep_pos) = decimal_sep {
        &last[..sep_pos]
    } else {
        last
    };
    if !last_int_part.is_empty() {
        integer_parts.push(last_int_part);
    }

    // Validate group sizes
    for (i, group) in integer_parts.iter().enumerate() {
        let len = group.len();
        if i == 0 {
            // First group: 1–3 digits
            if len == 0 || len > 3 {
                return Err(ParseError::InvalidGrouping);
            }
        } else {
            // Subsequent groups: exactly 3 digits
            if len != 3 {
                return Err(ParseError::InvalidGrouping);
            }
        }
    }

    // Rebuild without spaces
    let mut rebuilt = String::with_capacity(s.len());
    for part in &parts {
        rebuilt.push_str(part);
    }
    *s = rebuilt;

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers for character whitelist (step 5)
// ---------------------------------------------------------------------------

/// Check that `s` contains only ASCII digits, `.`, `,` and at least one digit.
fn check_whitelist_and_digits(s: &str) -> Result<(), ParseError> {
    let mut has_digit = false;
    for c in s.chars() {
        match c {
            '0'..='9' => has_digit = true,
            '.' | ',' => {}
            _ => return Err(ParseError::InvalidCharacter),
        }
    }
    if !has_digit {
        return Err(ParseError::MalformedNumber);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers for decimal-separator detection (step 6)
// ---------------------------------------------------------------------------

/// Result of decimal-separator disambiguation.
struct NumberParts {
    /// Integer digits as a string (group separators still present).
    integer: String,
    /// Fractional digits as a string.
    fraction: String,
}

/// Parse the decimal separator from `s`, splitting into integer and fractional
/// parts. Returns the result with group separators still present in the integer
/// part (they are removed in step 7).
fn detect_decimal(s: &str) -> Result<NumberParts, ParseError> {
    let dot_count = s.chars().filter(|&c| c == '.').count();
    let comma_count = s.chars().filter(|&c| c == ',').count();

    if dot_count == 0 && comma_count == 0 {
        // No separators — integer amount
        return Ok(NumberParts {
            integer: s.to_string(),
            fraction: String::new(),
        });
    }

    if dot_count > 0 && comma_count > 0 {
        // Both present — rightmost is decimal, the other is group
        let last_dot = s.rfind('.');
        let last_comma = s.rfind(',');

        let decimal_pos = match (last_dot, last_comma) {
            (Some(d), Some(c)) => std::cmp::max(d, c),
            (Some(d), None) => d,
            (None, Some(c)) => c,
            (None, None) => unreachable!(),
        };

        let group_sep = if decimal_pos == last_dot.unwrap_or(0) { ',' } else { '.' };

        // Check: all separators before decimal_pos must be the group separator
        let before_decimal = &s[..decimal_pos];
        for c in before_decimal.chars() {
            if c == '.' || c == ',' {
                if c != group_sep {
                    return Err(ParseError::InvalidGrouping);
                }
            }
        }

        // Check: no separator after decimal position
        let after_decimal = &s[decimal_pos + 1..];
        if after_decimal.contains('.') || after_decimal.contains(',') {
            return Err(ParseError::InvalidGrouping);
        }

        let integer = before_decimal.to_string();
        let fraction = after_decimal.to_string();
        return Ok(NumberParts { integer, fraction });
    }

    // Only one kind of separator present
    let sep = if dot_count > 0 { '.' } else { ',' };
    let count = dot_count.max(comma_count);
    let last_sep_pos = s.rfind(sep).unwrap();
    let digits_after = s.len() - last_sep_pos - 1;

    if count > 1 {
        // Multiple occurrences → group separator
        Ok(NumberParts {
            integer: s.to_string(),
            fraction: String::new(),
        })
    } else {
        // Exactly one occurrence — disambiguate by digits_after
        match digits_after {
            0 => Err(ParseError::MalformedNumber),
            1 | 2 => {
                // Decimal separator
                let (integer, fraction) = s.split_at(last_sep_pos);
                Ok(NumberParts {
                    integer: integer.to_string(),
                    fraction: fraction[1..].to_string(),
                })
            }
            3 => {
                // Ambiguity default: treat as group separator
                Ok(NumberParts {
                    integer: s.to_string(),
                    fraction: String::new(),
                })
            }
            _ => {
                // >= 4 digits after → InvalidGrouping
                Err(ParseError::InvalidGrouping)
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers for group/integer validation (step 7)
// ---------------------------------------------------------------------------

/// Validate digit group sizes and remove group separators.
/// TS001 §2.4 step 7: first group 1–3 digits, subsequent groups exactly 3.
/// Leading zero is only allowed if the integer part is exactly `"0"`.
fn validate_and_remove_groups(s: &str) -> Result<String, ParseError> {
    // Determine the group separator (if any) by counting '.' and ','
    let dot_count = s.chars().filter(|&c| c == '.').count();
    let comma_count = s.chars().filter(|&c| c == ',').count();

    let sep = if dot_count > 0 {
        '.'
    } else if comma_count > 0 {
        ','
    } else {
        // No group separators — just validate leading zero
        validate_leading_zero(s)?;
        return Ok(s.to_string());
    };

    let groups: Vec<&str> = s.split(sep).collect();

    if groups.iter().any(|g| g.is_empty()) {
        return Err(ParseError::InvalidGrouping);
    }

    for (i, group) in groups.iter().enumerate() {
        let len = group.len();
        if i == 0 {
            if len == 0 || len > 3 {
                return Err(ParseError::InvalidGrouping);
            }
        } else if len != 3 {
            return Err(ParseError::InvalidGrouping);
        }
    }

    // Validate leading zero on the reconstructed integer
    let digits: String = groups.concat();
    validate_leading_zero(&digits)?;

    Ok(digits)
}

fn validate_leading_zero(digits: &str) -> Result<(), ParseError> {
    if digits.len() > 1 && digits.starts_with('0') {
        return Err(ParseError::InvalidGrouping);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers for fraction handling (step 8) and assembly (step 9)
// ---------------------------------------------------------------------------

/// Right-pad fraction to exactly `exponent` digits. Rejects if too many
/// fractional digits and no rounding mode is set (Phase 1: always reject).
fn pad_fraction(
    fraction: &str,
    exponent: u8,
    _options: ParseOptions,
) -> Result<String, ParseError> {
    let e = exponent as usize;
    if fraction.len() > e {
        // Phase 1: rounding not implemented, always reject
        return Err(ParseError::TooManyFractionalDigits);
    }
    let mut padded = fraction.to_string();
    while padded.len() < e {
        padded.push('0');
    }
    Ok(padded)
}

/// Assemble the final minor units from integer and fraction digit strings.
/// Applies sign and checks for overflow (TS001 §2.4 step 9).
fn assemble_minor_units(
    integer_digits: &str,
    fraction_digits: &str,
    negative: bool,
) -> Result<i64, ParseError> {
    let digits = format!("{}{}", integer_digits, fraction_digits);

    // Parse as unsigned to handle i64::MIN edge case
    let unsigned: u64 = digits
        .parse()
        .map_err(|_| ParseError::Overflow)?;

    if negative {
        // Zero is canonical: -0 yields 0
        if unsigned == 0 {
            return Ok(0);
        }
        // Check that unsigned <= i64::MAX as u64 + 1 (for negation)
        if unsigned > (i64::MAX as u64) + 1 {
            return Err(ParseError::Overflow);
        }
        // Safe negation: handle i64::MIN case
        if unsigned == (i64::MAX as u64) + 1 {
            return Ok(i64::MIN);
        }
        Ok(-(unsigned as i64))
    } else {
        if unsigned > i64::MAX as u64 {
            return Err(ParseError::Overflow);
        }
        Ok(unsigned as i64)
    }
}

// ---------------------------------------------------------------------------
// Main parse implementation
// ---------------------------------------------------------------------------

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
        // Step 0: length guard
        if raw.chars().count() > 256 {
            return Err(ParseError::InputTooLong);
        }

        // Step 0: apply version-stable fold map
        let mut s: String = raw.chars().map(fold_char).collect();

        // Step 1: trim
        s = s.trim().to_string();
        if s.is_empty() {
            return Err(ParseError::EmptyInput);
        }

        // Step 2: sign extraction
        let negative = extract_sign(&mut s)?;

        // Step 3: currency-indicator extraction
        extract_currency_indicators(&mut s, currency)?;

        if s.is_empty() {
            return Err(ParseError::MalformedNumber);
        }

        // Step 4: validate and remove space groupings
        remove_space_groups(&mut s)?;

        // Step 5: character whitelist + digit presence
        check_whitelist_and_digits(&s)?;

        // Step 6: decimal-separator detection
        let number_parts = detect_decimal(&s)?;

        // Step 7: validate groups and remove group separators
        let integer_digits = validate_and_remove_groups(&number_parts.integer)?;

        // Step 8: fraction handling
        let fraction_digits = pad_fraction(&number_parts.fraction, currency.exponent(), options)?;

        // Step 9: assemble minor units
        let amount = assemble_minor_units(&integer_digits, &fraction_digits, negative)?;

        // Step 10: return Money
        Ok(Money::new(amount, currency))
    }
}
