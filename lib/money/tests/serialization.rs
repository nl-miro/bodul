//! Phase 1 serialization tests (TS001 §2.10, AC-S-1 … AC-S-10).

use money::{Currency, DeserializeError, Money};

#[test]
fn ac_s_1_serializes_canonical_form() {
    assert_eq!(
        Money::new(123456, Currency::USD).serialize(),
        r#"{ "amount_minor": "123456", "currency": "USD" }"#
    );
}

#[test]
fn ac_s_2_round_trips_beyond_json_safe_integer() {
    // 2^53 + 1 — would lose precision as a JSON number, exact as a string.
    let big = Money::new(9007199254740993, Currency::USD);
    let wire = big.serialize();
    assert_eq!(wire, r#"{ "amount_minor": "9007199254740993", "currency": "USD" }"#);
    assert_eq!(Money::deserialize(&wire), Ok(big));
}

#[test]
fn deserialize_round_trips_negative_and_zero() {
    for m in [
        Money::new(-1234, Currency::EUR),
        Money::new(0, Currency::CAD),
        Money::new(i64::MIN, Currency::AUD),
        Money::new(i64::MAX, Currency::USD),
    ] {
        assert_eq!(Money::deserialize(&m.serialize()), Ok(m));
    }
}

#[test]
fn ac_s_3_unknown_currency() {
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "123456", "currency": "XYZ" }"#),
        Err(DeserializeError::UnknownCurrency)
    );
}

#[test]
fn ac_s_4_number_form_amount_is_malformed_not_coerced() {
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": 123456, "currency": "USD" }"#),
        Err(DeserializeError::MalformedWireValue)
    );
}

#[test]
fn ac_s_5_through_10_invalid_amount_grammar() {
    let cases = [
        r#"{ "amount_minor": "12.5", "currency": "USD" }"#,  // AC-S-5 non-integer
        r#"{ "amount_minor": "007", "currency": "USD" }"#,   // AC-S-6 leading zero
        r#"{ "amount_minor": "+5", "currency": "USD" }"#,    // AC-S-8 leading plus
        r#"{ "amount_minor": "", "currency": "USD" }"#,      // AC-S-9 empty string
        r#"{ "amount_minor": " 5 ", "currency": "USD" }"#,   // AC-S-10 whitespace
    ];
    for wire in cases {
        assert_eq!(
            Money::deserialize(wire),
            Err(DeserializeError::InvalidAmountMinor),
            "wire {wire:?}"
        );
    }
    // `-0` is not canonical either.
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "-0", "currency": "USD" }"#),
        Err(DeserializeError::InvalidAmountMinor)
    );
}

#[test]
fn ac_s_7_out_of_range_amount() {
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "99999999999999999999", "currency": "USD" }"#),
        Err(DeserializeError::AmountOutOfRange)
    );
}

#[test]
fn missing_field_is_malformed() {
    assert_eq!(
        Money::deserialize(r#"{ "currency": "USD" }"#),
        Err(DeserializeError::MalformedWireValue)
    );
    assert_eq!(
        Money::deserialize(r#"{ "amount_minor": "5" }"#),
        Err(DeserializeError::MalformedWireValue)
    );
    assert_eq!(
        Money::deserialize("not json"),
        Err(DeserializeError::MalformedWireValue)
    );
}
