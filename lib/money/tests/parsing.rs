//! Phase 1 parsing tests (TS001 §2.4, positive baseline).
//!
//! Covers the Phase 1 AC-P baseline positives, AC-P-AMB ambiguity resolution,
//! AC-P-ZERO-3/4, the Phase 1 AC-P-NEG error rows (those not depending on
//! negative-amount *syntax*), and AC-NFR-1 (repeat-parse determinism).
//!
//! Negative amounts, accounting parentheses, and the full-width fold are Phase 2
//! and are intentionally not exercised here.

use money::{Currency, Money, ParseError, ParseOptions};

/// Parse with default options (rounding disabled).
fn parse(raw: &str, currency: Currency) -> Result<Money, ParseError> {
    Money::parse(raw, currency, ParseOptions::default())
}

/// Assert a successful parse to the given minor-unit amount.
fn assert_minor(raw: &str, currency: Currency, expected: i64) {
    assert_eq!(
        parse(raw, currency).map(|m| m.minor_units()),
        Ok(expected),
        "input {raw:?}"
    );
}

// ----- AC-P · positive baseline (Phase 1 rows) -----

#[test]
fn ac_p_usd_baseline() {
    assert_minor("$1,234.56", Currency::USD, 123456); // AC-P-1 (with indicator)
    assert_minor("US$12.30", Currency::USD, 1230); // AC-P-3 (with indicator)
    assert_minor("1234.56", Currency::USD, 123456); // AC-P-2 (no indicator)
}

#[test]
fn ac_p_eur_baseline() {
    assert_minor("1.234,56 €", Currency::EUR, 123456); // AC-P-7 (Family B, trailing €)
    assert_minor("€1,234.56", Currency::EUR, 123456); // AC-P-10 (Family A, Ireland)
    assert_minor("1234,56", Currency::EUR, 123456); // AC-P-32 (no indicator)
}

#[test]
fn ac_p_eur_space_grouped() {
    // AC-P-8: NBSP before €. AC-P-9: narrow no-break space as the group separator.
    assert_minor("1.234,56\u{00A0}€", Currency::EUR, 123456);
    assert_minor("1\u{202F}234,56 €", Currency::EUR, 123456);
}

#[test]
fn ac_p_cad_baseline() {
    assert_minor("$1,234.56", Currency::CAD, 123456); // AC-P-14 (bare $, valid for CAD)
    assert_minor("CA$1,234.56", Currency::CAD, 123456); // AC-P-15
    assert_minor("1 234,56 $", Currency::CAD, 123456); // AC-P-16 (fr-CA, trailing $)
    assert_minor("1234.56", Currency::CAD, 123456); // AC-P-33 (no indicator)
}

#[test]
fn ac_p_aud_baseline() {
    assert_minor("A$1,234.56", Currency::AUD, 123456); // AC-P-17
    assert_minor("AUD 12.30", Currency::AUD, 1230); // AC-P-18
    assert_minor("1234.56", Currency::AUD, 123456); // AC-P-34 (no indicator)
}

// ----- AC-P-AMB · ambiguity resolution (default options) -----

#[test]
fn ac_p_amb_grouping_vs_decimal() {
    assert_minor("1,234", Currency::USD, 123400); // AC-P-AMB-1 (group)
    assert_minor("1.234", Currency::EUR, 123400); // AC-P-AMB-2 (group)
    assert_minor("1.23", Currency::USD, 123); // AC-P-AMB-3 (decimal)
    assert_minor("1,23", Currency::EUR, 123); // AC-P-AMB-4 (decimal)
    assert_minor("12.345", Currency::USD, 1234500); // AC-P-AMB-5 (group)
    assert_minor("1.234.567,89", Currency::EUR, 123456789); // AC-P-AMB-6
    assert_minor("1.000", Currency::EUR, 100000); // AC-P-AMB-7 (group)
}

#[test]
fn ac_p_large_grouped_value() {
    assert_minor("1.000.000,00 €", Currency::EUR, 100000000); // AC-P-13
}

#[test]
fn ac_p_26_full_width_characters() {
    // AC-P-26: full-width dollar, digits, and full stop fold to ASCII in step 0.
    assert_minor("＄１２．３０", Currency::USD, 1230);
}

#[test]
fn can_dollar_indicator_is_recognised() {
    // TS001 §2.2 lists `Can$` as a CAD indicator; longest-match must prefer it over
    // the shorter `CA$`.
    assert_minor("Can$5.00", Currency::CAD, 500);
}

// ----- AC-P-ZERO · positive zero (Phase 1 rows) -----

#[test]
fn ac_p_zero_positive() {
    assert_eq!(
        parse("0,50 €", Currency::EUR),
        Ok(Money::new(50, Currency::EUR))
    ); // AC-P-ZERO-3
    assert_eq!(
        parse("0.99", Currency::USD),
        Ok(Money::new(99, Currency::USD))
    ); // AC-P-ZERO-4
}

// ----- AC-P-NEG · error rows reachable in Phase 1 -----

#[test]
fn ac_p_neg_empty_and_invalid_chars() {
    assert_eq!(parse("", Currency::USD), Err(ParseError::EmptyInput)); // NEG-1
    assert_eq!(
        parse("abc", Currency::USD),
        Err(ParseError::InvalidCharacter)
    ); // NEG-2
    assert_eq!(
        parse("1'234.56", Currency::USD),
        Err(ParseError::InvalidCharacter)
    ); // NEG-8 (Swiss)
}

#[test]
fn ac_p_neg_currency_mismatch() {
    assert_eq!(
        parse("$5.00", Currency::EUR),
        Err(ParseError::CurrencyMismatch)
    ); // NEG-3
    assert_eq!(
        parse("€5,00", Currency::USD),
        Err(ParseError::CurrencyMismatch)
    ); // NEG-4
    assert_eq!(
        parse("US$5.00", Currency::CAD),
        Err(ParseError::CurrencyMismatch)
    ); // NEG-20
}

#[test]
fn ac_p_neg_invalid_grouping() {
    assert_eq!(
        parse("1.2.3", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-5
    assert_eq!(
        parse("1,23,456", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-6
    assert_eq!(
        parse("12,34,567", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-7 (Indian)
    assert_eq!(
        parse("1.0000", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-15
    assert_eq!(
        parse("1,2345", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-16
    assert_eq!(
        parse("1,234.567,89", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-17
    assert_eq!(parse("00", Currency::USD), Err(ParseError::InvalidGrouping)); // NEG-18
    assert_eq!(
        parse("007", Currency::USD),
        Err(ParseError::InvalidGrouping)
    ); // NEG-19
    assert_eq!(
        parse("12\u{00A0}34,56 €", Currency::EUR),
        Err(ParseError::InvalidGrouping)
    ); // NEG-22 (bad space group)
}

#[test]
fn too_many_fractional_digits_is_rejected() {
    // A genuinely-fractional input with >2 decimal digits is rejected in the default
    // (reject) mode. Both separators present makes the rightmost (`,`) unambiguously
    // the decimal separator, so `567` is a 3-digit fraction → TooManyFractionalDigits.
    //
    // NOTE: AC-P-NEG-9 (`12.999` → TooManyFractionalDigits) is NOT reproduced here.
    // A lone separator before exactly three digits is grouping by the §2.4 step-6
    // ambiguity default (Decision #3, `decided`), identical to AC-P-AMB-5
    // (`12.345` → 1234500). See breakingchanges-claude.md.
    assert_eq!(
        parse("1.234,567", Currency::USD),
        Err(ParseError::TooManyFractionalDigits)
    );
    // `12.999` therefore parses as grouping, consistent with AC-P-AMB-5.
    assert_minor("12.999", Currency::USD, 1299900);
}

#[test]
fn ac_p_neg_overflow() {
    assert_eq!(
        parse("99999999999999999999", Currency::USD),
        Err(ParseError::Overflow)
    ); // NEG-10
}

#[test]
fn ac_p_neg_malformed_number() {
    assert_eq!(parse("1,", Currency::USD), Err(ParseError::MalformedNumber)); // NEG-11
    assert_eq!(parse("$", Currency::USD), Err(ParseError::MalformedNumber)); // NEG-23
    assert_eq!(
        parse("USD", Currency::USD),
        Err(ParseError::MalformedNumber)
    ); // NEG-24
}

#[test]
fn ac_p_neg_malformed_currency() {
    assert_eq!(
        parse("$1,234.56 USD", Currency::USD),
        Err(ParseError::MalformedCurrency)
    ); // NEG-12
    assert_eq!(
        parse("US$ 12.34 USD", Currency::USD),
        Err(ParseError::MalformedCurrency)
    ); // NEG-13
    assert_eq!(
        parse("1.234,56 € EUR", Currency::EUR),
        Err(ParseError::MalformedCurrency)
    ); // NEG-14
    assert_eq!(
        parse("CAD$5.00 CAD", Currency::CAD),
        Err(ParseError::MalformedCurrency)
    ); // NEG-26
}

#[test]
fn ac_p_neg_input_too_long() {
    // NEG-21: input longer than 256 Unicode scalar values.
    let long = "1".repeat(257);
    assert_eq!(parse(&long, Currency::USD), Err(ParseError::InputTooLong));
    // At the 256-char boundary the guard does not fire (the value itself overflows
    // i64, but that is a different, later error — the point is it is not rejected as
    // too long).
    let at_limit = "1".repeat(256);
    assert_ne!(
        parse(&at_limit, Currency::USD),
        Err(ParseError::InputTooLong)
    );
}

// ----- AC-NFR-1 · determinism -----

#[test]
fn ac_nfr_1_repeat_parse_is_deterministic() {
    let inputs = ["$1,234.56", "1.234,56 €", "1 234,56 $"];
    let currencies = [Currency::USD, Currency::EUR, Currency::CAD];
    for (raw, currency) in inputs.iter().zip(currencies) {
        let first = parse(raw, currency);
        let second = parse(raw, currency);
        assert_eq!(first, second, "input {raw:?}");
    }
}
