//! Phase 1 construction, accessors, and structural equality/hash tests.
//!
//! Covers TS001 §2.2 (`Currency::exponent`), §2.9 (`new`, `from_major`,
//! `minor_units`, `currency`), the AC-A structural-equality/hash rows, AC-A-18/19,
//! and AC-NFR-3 (operations do not mutate inputs).

use money::{Currency, Money, MoneyError};

#[test]
fn currency_exponent_is_two_for_all_supported_currencies() {
    // TS001 §2.2: every supported currency is exponent-2.
    for currency in [Currency::USD, Currency::EUR, Currency::CAD, Currency::AUD] {
        assert_eq!(currency.exponent(), 2);
    }
}

#[test]
fn new_and_accessors_round_trip() {
    let m = Money::new(123456, Currency::USD);
    assert_eq!(m.minor_units(), 123456);
    assert_eq!(m.currency(), Currency::USD);

    // Negative and zero are valid minor-unit values (signed storage).
    assert_eq!(Money::new(-500, Currency::EUR).minor_units(), -500);
    assert_eq!(Money::new(0, Currency::CAD).minor_units(), 0);
}

#[test]
fn equality_and_hash_are_total_over_amount_and_currency() {
    use std::collections::HashSet;

    // Same amount, different currency → not equal (TS001 §2.6, INV-3).
    assert_ne!(Money::new(500, Currency::USD), Money::new(500, Currency::EUR));
    assert_eq!(Money::new(500, Currency::USD), Money::new(500, Currency::USD));

    // Safe as a hash key: the two distinct values coexist in a set.
    let mut set = HashSet::new();
    set.insert(Money::new(500, Currency::USD));
    set.insert(Money::new(500, Currency::EUR));
    set.insert(Money::new(500, Currency::USD)); // duplicate of the first
    assert_eq!(set.len(), 2);
}

#[test]
fn from_major_combines_units_and_fraction() {
    // AC-A-18.
    assert_eq!(
        Money::from_major(-12, -34, Currency::USD),
        Ok(Money::new(-1234, Currency::USD))
    );
    assert_eq!(
        Money::from_major(0, -34, Currency::USD),
        Ok(Money::new(-34, Currency::USD))
    );

    // Positive and zero baselines.
    assert_eq!(
        Money::from_major(12, 34, Currency::USD),
        Ok(Money::new(1234, Currency::USD))
    );
    assert_eq!(
        Money::from_major(5, 0, Currency::EUR),
        Ok(Money::new(500, Currency::EUR))
    );
}

#[test]
fn from_major_rejects_mixed_signs_and_oversized_fraction() {
    // AC-A-19.
    assert_eq!(
        Money::from_major(12, -34, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
    assert_eq!(
        Money::from_major(12, 100, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
    // Mirror sign case: negative units with positive fraction.
    assert_eq!(
        Money::from_major(-12, 34, Currency::USD),
        Err(MoneyError::InvalidArgument)
    );
}

#[test]
fn from_major_is_overflow_checked() {
    // units * 100 overflows i64.
    assert_eq!(
        Money::from_major(i64::MAX, 0, Currency::USD),
        Err(MoneyError::Overflow)
    );
}

#[test]
fn operations_do_not_mutate_inputs() {
    // AC-NFR-3: accessors and constructors return values without mutating inputs.
    let original = Money::new(777, Currency::AUD);
    let _ = original.minor_units();
    let _ = original.currency();
    let _ = Money::from_major(1, 0, Currency::AUD);
    assert_eq!(original, Money::new(777, Currency::AUD));
}
