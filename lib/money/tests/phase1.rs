use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use money::{
    Currency, DeserializeError, Money, MoneyError, ParseError, ParseOptions, RoundingMode,
};

fn parse(input: &str, currency: Currency) -> Result<i64, ParseError> {
    Money::parse(input, currency, ParseOptions::default()).map(|money| money.minor_units())
}

#[test]
fn constructs_and_inspects_money_values() {
    let money = Money::new(500, Currency::USD);

    assert_eq!(money.minor_units(), 500);
    assert_eq!(money.currency(), Currency::USD);
    assert_eq!(Currency::USD.exponent(), 2);
    assert_eq!(Currency::EUR.exponent(), 2);
    assert_eq!(Currency::CAD.exponent(), 2);
    assert_eq!(Currency::AUD.exponent(), 2);
    assert_ne!(
        Money::new(500, Currency::USD),
        Money::new(500, Currency::EUR)
    );

    let mut usd_hasher = DefaultHasher::new();
    Money::new(500, Currency::USD).hash(&mut usd_hasher);
    let mut eur_hasher = DefaultHasher::new();
    Money::new(500, Currency::EUR).hash(&mut eur_hasher);
    assert_ne!(usd_hasher.finish(), eur_hasher.finish());
}

#[test]
fn from_major_validates_sign_magnitude_and_overflow() {
    assert_eq!(
        Money::from_major(-12, -34, Currency::USD),
        Ok(Money::new(-1234, Currency::USD))
    );
    assert_eq!(
        Money::from_major(0, -34, Currency::USD),
        Ok(Money::new(-34, Currency::USD))
    );
    assert_eq!(
        Money::from_major(12, -34, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
    assert_eq!(
        Money::from_major(12, 100, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
    assert_eq!(
        Money::from_major(i64::MAX, 0, Currency::USD),
        Err(MoneyError::Overflow)
    );
}

#[test]
fn parses_phase1_positive_baseline_inputs() {
    let cases = [
        (Currency::USD, "$1,234.56", 123456),
        (Currency::USD, "1234.56", 123456),
        (Currency::USD, "US$12.30", 1230),
        (Currency::USD, "USD 0.99", 99),
        (Currency::USD, "5", 500),
        (Currency::USD, "5.5", 550),
        (Currency::EUR, "1.234,56 €", 123456),
        (Currency::EUR, "1.234,56\u{00a0}€", 123456),
        (Currency::EUR, "1\u{202f}234,56 €", 123456),
        (Currency::EUR, "€1,234.56", 123456),
        (Currency::EUR, "€ 1.234,56", 123456),
        (Currency::EUR, "0,99 €", 99),
        (Currency::EUR, "1.000.000,00 €", 100000000),
        (Currency::CAD, "$1,234.56", 123456),
        (Currency::CAD, "CA$1,234.56", 123456),
        (Currency::CAD, "1 234,56 $", 123456),
        (Currency::AUD, "A$1,234.56", 123456),
        (Currency::AUD, "AUD 12.30", 1230),
        (Currency::CAD, "CAD$5.00", 500),
        (Currency::USD, "US$5.00", 500),
        (Currency::EUR, "1234,56", 123456),
        (Currency::CAD, "1234.56", 123456),
        (Currency::AUD, "1234.56", 123456),
        (Currency::USD, "＄１２．３０", 1230),
    ];

    for (currency, input, expected) in cases {
        assert_eq!(parse(input, currency), Ok(expected), "{input}");
    }
}

#[test]
fn parses_ambiguous_separators_and_zero_values() {
    let cases = [
        (Currency::USD, "1,234", 123400),
        (Currency::EUR, "1.234", 123400),
        (Currency::USD, "1.23", 123),
        (Currency::EUR, "1,23", 123),
        (Currency::USD, "12.345", 1234500),
        (Currency::USD, "12.998", 1299800),
        (Currency::USD, "12.999", 1299900),
        (Currency::USD, "13.999", 1399900),
        (Currency::EUR, "1.234.567,89", 123456789),
        (Currency::EUR, "1.000", 100000),
        (Currency::EUR, "0,50 €", 50),
        (Currency::USD, "0.99", 99),
    ];

    for (currency, input, expected) in cases {
        assert_eq!(parse(input, currency), Ok(expected), "{input}");
    }
}

#[test]
fn returns_typed_parse_errors() {
    let too_long = "1".repeat(257);
    let cases = [
        (Currency::USD, "", ParseError::EmptyInput),
        (Currency::USD, "abc", ParseError::InvalidCharacter),
        (Currency::EUR, "$5.00", ParseError::CurrencyMismatch),
        (Currency::USD, "€5,00", ParseError::CurrencyMismatch),
        (Currency::USD, "1.2.3", ParseError::InvalidGrouping),
        (Currency::USD, "1,23,456", ParseError::InvalidGrouping),
        (Currency::USD, "12,34,567", ParseError::InvalidGrouping),
        (Currency::USD, "1'234.56", ParseError::InvalidCharacter),
        (Currency::USD, "99999999999999999999", ParseError::Overflow),
        (Currency::USD, "1,", ParseError::MalformedNumber),
        (
            Currency::USD,
            "$1,234.56 USD",
            ParseError::MalformedCurrency,
        ),
        (
            Currency::USD,
            "US$ 12.34 USD",
            ParseError::MalformedCurrency,
        ),
        (
            Currency::EUR,
            "1.234,56 € EUR",
            ParseError::MalformedCurrency,
        ),
        (Currency::USD, "1.0000", ParseError::InvalidGrouping),
        (Currency::USD, "1,2345", ParseError::InvalidGrouping),
        (Currency::USD, "1,234.567,89", ParseError::InvalidGrouping),
        (Currency::USD, "00", ParseError::InvalidGrouping),
        (Currency::USD, "007", ParseError::InvalidGrouping),
        (Currency::CAD, "US$5.00", ParseError::CurrencyMismatch),
        (Currency::EUR, "12 34,56 €", ParseError::InvalidGrouping),
        (Currency::USD, "$", ParseError::MalformedNumber),
        (Currency::USD, "USD", ParseError::MalformedNumber),
        (Currency::USD, "-$", ParseError::MalformedNumber),
        (Currency::CAD, "CAD$5.00 CAD", ParseError::MalformedCurrency),
        (Currency::CAD, "$5.00 CAD", ParseError::MalformedCurrency),
        (Currency::USD, "$5.00 USD", ParseError::MalformedCurrency),
        (Currency::AUD, "$5.00 AUD", ParseError::MalformedCurrency),
    ];

    for (currency, input, expected) in cases {
        assert_eq!(
            Money::parse(input, currency, ParseOptions::default()),
            Err(expected),
            "{input}"
        );
    }
    assert_eq!(
        Money::parse(&too_long, Currency::USD, ParseOptions::default()),
        Err(ParseError::InputTooLong)
    );
}

#[test]
fn keeps_phase2_negative_and_rounding_behavior_out() {
    assert_eq!(
        Money::parse("-$5.00", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedSign)
    );
    assert_eq!(
        Money::parse("($5.00)", Currency::USD, ParseOptions::default()),
        Err(ParseError::MalformedSign)
    );
    assert_eq!(
        Money::parse(
            "12.999",
            Currency::USD,
            ParseOptions {
                rounding: Some(RoundingMode::HalfUp),
            },
        ),
        Ok(Money::new(1299900, Currency::USD))
    );
}

#[test]
fn serializes_and_deserializes_canonical_wire_values() {
    assert_eq!(
        Money::new(123456, Currency::USD).serialize(),
        r#"{ "amount_minor": "123456", "currency": "USD" }"#
    );

    let large = Money::new(9_007_199_254_740_993, Currency::USD);
    assert_eq!(Money::deserialize(&large.serialize()), Ok(large));
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "123456", "currency": "XYZ" }"#),
        Err(DeserializeError::UnknownCurrency)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": 123456, "currency": "USD" }"#),
        Err(DeserializeError::MalformedWireValue)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "12.5", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "007", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "99999999999999999999", "currency": "USD" }"#),
        Err(DeserializeError::AmountOutOfRange)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "+5", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": " 5 ", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}
