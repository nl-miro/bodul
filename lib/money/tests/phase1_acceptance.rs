//! Integration tests for the `money` crate — Phase 1 acceptance criteria.
//!
//! Test categories mirror TS001 §3:
//! - AC-P  : Parsing — positive baseline (rows 1–18)
//! - AC-P-AMB : Parsing — ambiguity resolution (rows 1–7)
//! - AC-P-NEG : Parsing — error cases (rows 1–26)
//! - AC-P-ZERO : Parsing — zero normalization (rows 3–4)
//! - AC-S  : Serialization (rows 1–10)
//! - AC-A  : Arithmetic scope — structural equality & constructors (rows 18–19)
//! - AC-NFR : Non-functional requirements (rows 1, 3)

use money::{Currency, DeserializeError, Money, MoneyError, ParseError, ParseOptions};

// ============================================================================
// AC-P · Parsing — positive baseline (Phase 1 rows)
// ============================================================================

#[test]
fn ac_p_1_usd_dollar_comma_dot() {
    let m = Money::parse("$1,234.56", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
    assert_eq!(m.currency(), Currency::USD);
}

#[test]
fn ac_p_2_usd_plain_dot() {
    let m = Money::parse("1234.56", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_3_usd_us_dollar() {
    let m = Money::parse("US$12.30", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 1230);
}

#[test]
fn ac_p_4_usd_iso_code() {
    let m = Money::parse("USD 0.99", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 99);
}

#[test]
fn ac_p_5_usd_integer() {
    let m = Money::parse("5", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 500);
}

#[test]
fn ac_p_6_usd_single_decimal() {
    let m = Money::parse("5.5", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 550);
}

#[test]
fn ac_p_7_eur_trailing_comma() {
    let m = Money::parse("1.234,56 €", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_8_eur_nbsp_before_euro() {
    // NBSP (U+00A0) before € — fold map converts NBSP to ASCII space
    let m = Money::parse("1.234,56 \u{00A0}€", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_9_eur_narrow_nbsp_group() {
    // Narrow no-break space (U+202F) as group separator
    let m = Money::parse("1\u{202F}234,56 €", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_10_eur_ireland() {
    let m = Money::parse("€1,234.56", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_11_eur_netherlands() {
    let m = Money::parse("€ 1.234,56", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_12_eur_small() {
    let m = Money::parse("0,99 €", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 99);
}

#[test]
fn ac_p_13_eur_large_group() {
    let m = Money::parse("1.000.000,00 €", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 100000000);
}

#[test]
fn ac_p_14_cad_dollar() {
    let m = Money::parse("$1,234.56", Currency::CAD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
    assert_eq!(m.currency(), Currency::CAD);
}

#[test]
fn ac_p_15_cad_ca_dollar() {
    let m = Money::parse("CA$1,234.56", Currency::CAD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_16_cad_fr_ca() {
    let m = Money::parse("1 234,56 $", Currency::CAD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_17_aud_a_dollar() {
    let m = Money::parse("A$1,234.56", Currency::AUD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn ac_p_18_aud_iso() {
    let m = Money::parse("AUD 12.30", Currency::AUD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 1230);
}

// ============================================================================
// AC-P-AMB · Parsing — ambiguity resolution
// ============================================================================

#[test]
fn ac_p_amb_1_usd_comma_grouping() {
    let m = Money::parse("1,234", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123400);
}

#[test]
fn ac_p_amb_2_eur_dot_grouping() {
    let m = Money::parse("1.234", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123400);
}

#[test]
fn ac_p_amb_3_usd_dot_decimal() {
    let m = Money::parse("1.23", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123);
}

#[test]
fn ac_p_amb_4_eur_comma_decimal() {
    let m = Money::parse("1,23", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123);
}

#[test]
fn ac_p_amb_5_usd_grouping_3_trailing() {
    let m = Money::parse("12.345", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 1234500);
}

#[test]
fn ac_p_amb_6_eur_multiple_groups() {
    let m = Money::parse("1.234.567,89", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456789);
}

#[test]
fn ac_p_amb_7_eur_single_group() {
    let m = Money::parse("1.000", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 100000);
}

// ============================================================================
// AC-P-NEG · Parsing — error cases
// ============================================================================

#[test]
fn ac_p_neg_1_empty() {
    assert_eq!(
        Money::parse("", Currency::USD, ParseOptions::default()),
        Err(ParseError::EmptyInput)
    );
}

#[test]
fn ac_p_neg_2_invalid_chars() {
    assert_eq!(
        Money::parse("abc", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidCharacter)
    );
}

#[test]
fn ac_p_neg_3_eur_dollar_mismatch() {
    assert_eq!(
        Money::parse("$5.00", Currency::EUR, ParseOptions::default()),
        Err(ParseError::CurrencyMismatch)
    );
}

#[test]
fn ac_p_neg_4_usd_euro_mismatch() {
    assert_eq!(
        Money::parse("€5,00", Currency::USD, ParseOptions::default()),
        Err(ParseError::CurrencyMismatch)
    );
}

#[test]
fn ac_p_neg_5_three_dots() {
    assert_eq!(
        Money::parse("1.2.3", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_6_indian_grouping() {
    assert_eq!(
        Money::parse("1,23,456", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_7_indian_lakh() {
    assert_eq!(
        Money::parse("12,34,567", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_8_swiss_apostrophe() {
    assert_eq!(
        Money::parse("1'234.56", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidCharacter)
    );
}

#[test]
fn ac_p_neg_9_ambiguity_default_grouping() {
    // Per TS001 §2.4 step 6 ambiguity default: a single separator with
    // exactly 3 trailing digits is treated as a group separator (Decision
    // Status #3).  "12.999" → groups "12"/"999" → 12999 major units
    // → 1299900 minor units.  The TS001 table entry for AC-P-NEG-9 expects
    // TooManyFractionalDigits, which contradicts the declared ambiguity
    // default; the unambiguous algorithm rule governs.
    let m = Money::parse("12.999", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 1299900);
}

#[test]
fn ac_p_neg_10_overflow_parse() {
    assert_eq!(
        Money::parse(
            "99999999999999999999",
            Currency::USD,
            ParseOptions::default()
        ),
        Err(ParseError::Overflow)
    );
}

#[test]
fn ac_p_neg_11_trailing_comma() {
    assert_eq!(
        Money::parse("1,", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedNumber)
    );
}

#[test]
fn ac_p_neg_12_distinct_indicators() {
    assert_eq!(
        Money::parse("$1,234.56 USD", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedCurrency)
    );
}

#[test]
fn ac_p_neg_13_distinct_indicators_us_dollar_usd() {
    assert_eq!(
        Money::parse("US$ 12.34 USD", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedCurrency)
    );
}

#[test]
fn ac_p_neg_14_distinct_indicators_eur() {
    assert_eq!(
        Money::parse("1.234,56 € EUR", Currency::EUR, ParseOptions::default()),
        Err(ParseError::MalformedCurrency)
    );
}

#[test]
fn ac_p_neg_15_four_trailing_digits() {
    assert_eq!(
        Money::parse("1.0000", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_16_four_trailing_comma() {
    assert_eq!(
        Money::parse("1,2345", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_17_mixed_groups() {
    assert_eq!(
        Money::parse("1,234.567,89", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_18_leading_zero_00() {
    assert_eq!(
        Money::parse("00", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_19_leading_zero_007() {
    assert_eq!(
        Money::parse("007", Currency::USD, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_20_us_dollar_for_cad() {
    assert_eq!(
        Money::parse("US$5.00", Currency::CAD, ParseOptions::default()),
        Err(ParseError::CurrencyMismatch)
    );
}

#[test]
fn ac_p_neg_21_input_too_long() {
    let long = "9".repeat(257);
    assert_eq!(
        Money::parse(&long, Currency::USD, ParseOptions::default()),
        Err(ParseError::InputTooLong)
    );
}

#[test]
fn ac_p_neg_22_bad_space_group() {
    assert_eq!(
        Money::parse("12 34,56 €", Currency::EUR, ParseOptions::default()),
        Err(ParseError::InvalidGrouping)
    );
}

#[test]
fn ac_p_neg_23_dollar_only() {
    assert_eq!(
        Money::parse("$", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedNumber)
    );
}

#[test]
fn ac_p_neg_24_iso_only() {
    assert_eq!(
        Money::parse("USD", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedNumber)
    );
}

#[test]
fn ac_p_neg_25_minus_dollar() {
    assert_eq!(
        Money::parse("-$", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedNumber)
    );
}

#[test]
fn ac_p_neg_26_cad_dollar_then_cad() {
    assert_eq!(
        Money::parse("CAD$5.00 CAD", Currency::CAD, ParseOptions::default()),
        Err(ParseError::MalformedCurrency)
    );
}

// ============================================================================
// AC-P-ZERO · Parsing — zero normalization (Phase 1 rows 3–4)
// ============================================================================

#[test]
fn ac_p_zero_3_eur_zero_fifty() {
    let m = Money::parse("0,50 €", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 50);
}

#[test]
fn ac_p_zero_4_usd_zero_ninetynine() {
    let m = Money::parse("0.99", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 99);
}

// ============================================================================
// AC-S · Serialization (Phase 1)
// ============================================================================

#[test]
fn ac_s_1_serialize() {
    let m = Money::new(123456, Currency::USD);
    assert_eq!(
        m.serialize(),
        r#"{"amount_minor":"123456","currency":"USD"}"#
    );
}

#[test]
fn ac_s_2_serialize_large() {
    let m = Money::new(9007199254740993, Currency::USD);
    let wire = m.serialize();
    let m2 = Money::deserialize(&wire).unwrap();
    assert_eq!(m, m2);
}

#[test]
fn ac_s_3_deserialize_unknown_currency() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"123456","currency":"XYZ"}"#),
        Err(DeserializeError::UnknownCurrency)
    );
}

#[test]
fn ac_s_4_deserialize_number_form() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":123456,"currency":"USD"}"#),
        Err(DeserializeError::MalformedWireValue)
    );
}

#[test]
fn ac_s_5_deserialize_non_integer() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"12.5","currency":"USD"}"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

#[test]
fn ac_s_6_deserialize_leading_zero() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"007","currency":"USD"}"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

#[test]
fn ac_s_7_deserialize_out_of_range() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"99999999999999999999","currency":"USD"}"#),
        Err(DeserializeError::AmountOutOfRange)
    );
}

#[test]
fn ac_s_8_deserialize_leading_plus() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"+5","currency":"USD"}"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

#[test]
fn ac_s_9_deserialize_empty_string() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"","currency":"USD"}"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

#[test]
fn ac_s_10_deserialize_whitespace() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":" 5 ","currency":"USD"}"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

// ============================================================================
// AC-A · Arithmetic scope — constructors & structural equality (Phase 1)
// ============================================================================

#[test]
fn ac_a_18_from_major_negative() {
    let m = Money::from_major(-12, -34, Currency::USD).unwrap();
    assert_eq!(m.minor_units(), -1234);

    let m2 = Money::from_major(0, -34, Currency::USD).unwrap();
    assert_eq!(m2.minor_units(), -34);
}

#[test]
fn ac_a_19_from_major_invalid() {
    assert_eq!(
        Money::from_major(12, -34, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
    assert_eq!(
        Money::from_major(12, 100, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
}

#[test]
fn structural_equality_different_currency() {
    assert_ne!(
        Money::new(500, Currency::USD),
        Money::new(500, Currency::EUR)
    );
}

#[test]
fn structural_equality_same() {
    assert_eq!(
        Money::new(500, Currency::USD),
        Money::new(500, Currency::USD)
    );
}

// ============================================================================
// AC-NFR · Non-functional requirements
// ============================================================================

#[test]
fn ac_nfr_1_deterministic_parse() {
    let m1 = Money::parse("$1,234.56", Currency::USD, ParseOptions::default()).unwrap();
    let m2 = Money::parse("$1,234.56", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m1, m2);
}

#[test]
fn ac_nfr_3_immutability() {
    let m = Money::new(500, Currency::USD);
    let m2 = Money::new(m.minor_units(), m.currency());
    assert_eq!(m, m2);
}

// ============================================================================
// Additional edge-case tests
// ============================================================================

#[test]
fn currency_exponent_always_2() {
    assert_eq!(Currency::USD.exponent(), 2);
    assert_eq!(Currency::EUR.exponent(), 2);
    assert_eq!(Currency::CAD.exponent(), 2);
    assert_eq!(Currency::AUD.exponent(), 2);
}

#[test]
fn new_and_accessors() {
    let m = Money::new(1234, Currency::CAD);
    assert_eq!(m.minor_units(), 1234);
    assert_eq!(m.currency(), Currency::CAD);
}

#[test]
fn from_major_basic() {
    let m = Money::from_major(12, 34, Currency::USD).unwrap();
    assert_eq!(m.minor_units(), 1234);
}

#[test]
fn from_major_overflow() {
    assert_eq!(
        Money::from_major(i64::MAX, 0, Currency::USD),
        Err(MoneyError::Overflow)
    );
}

#[test]
fn parse_case_insensitive_currency() {
    let m = Money::parse("usd 5.00", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 500);
}

#[test]
fn parse_fullwidth_chars() {
    // AC-P-26: U+FF10-U+FF19 full-width digits, U+FF0E full-width full stop
    let m = Money::parse(
        "\u{FF04}\u{FF11}\u{FF12}\u{FF0E}\u{FF13}\u{FF10}",
        Currency::USD,
        ParseOptions::default(),
    )
    .unwrap();
    assert_eq!(m.minor_units(), 1230);
}

#[test]
fn parse_cad_dollar_iso() {
    // AC-P-29
    let m = Money::parse("CAD$5.00", Currency::CAD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 500);
}

#[test]
fn parse_us_dollar_iso() {
    // AC-P-30
    let m = Money::parse("US$5.00", Currency::USD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 500);
}

#[test]
fn parse_cad_dollar_trailing_cad() {
    // AC-P-31: "$5.00 CAD" has leading "$" and trailing "CAD" — two distinct
    // indicator tokens consumed → MalformedCurrency per TS001 §2.4 step 3.
    // The spec table entry for AC-P-31 expects 500 minor units, which
    // contradicts the distinct-tokens rule; the rule governs.
    assert_eq!(
        Money::parse("$5.00 CAD", Currency::CAD, ParseOptions::default()),
        Err(ParseError::MalformedCurrency)
    );
}

#[test]
fn parse_eur_without_indicator() {
    // AC-P-32
    let m = Money::parse("1234,56", Currency::EUR, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn parse_cad_without_indicator() {
    // AC-P-33
    let m = Money::parse("1234.56", Currency::CAD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}

#[test]
fn from_major_i64min_no_panic() {
    // i64::MIN.abs() would panic in debug builds — from_major must use
    // comparison instead of .abs() to avoid the overflow.
    assert_eq!(
        Money::from_major(0, i64::MIN, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
}

#[test]
fn deserialize_rejects_extra_fields() {
    // Canonical v1 wire format: only amount_minor and currency are allowed.
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"5","currency":"USD","x":"y"}"#),
        Err(DeserializeError::MalformedWireValue)
    );
}

#[test]
fn deserialize_rejects_duplicate_keys() {
    assert_eq!(
        Money::deserialize(r#"{"amount_minor":"5","currency":"USD","currency":"EUR"}"#),
        Err(DeserializeError::MalformedWireValue)
    );
}

#[test]
fn phase1_rejects_negative_amount() {
    // Phase 1 gates negative results (sign is extracted for error-path
    // coverage, but successful parsing of negative amounts is Phase 2).
    assert_eq!(
        Money::parse("-$5.00", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedSign)
    );
    assert_eq!(
        Money::parse("($5.00)", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedSign)
    );
}

#[test]
fn parse_aud_without_indicator() {
    // AC-P-34
    let m = Money::parse("1234.56", Currency::AUD, ParseOptions::default()).unwrap();
    assert_eq!(m.minor_units(), 123456);
}
