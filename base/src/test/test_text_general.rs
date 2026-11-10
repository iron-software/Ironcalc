#![allow(clippy::unwrap_used)]

// IronXL: TEXT with General gives the Excel text, with at most 11 characters without the sign.
// The rounding is half away from zero, and a mantissa that rounds to 10 moves the exponent.

use crate::test::util::new_empty_model;

fn assert_results(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("A{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("A{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn text_general_shows_at_most_eleven_characters() {
    assert_results(&[
        ("=TEXT(1234567890.5,\"General\")", "1234567891"),
        ("=TEXT(12345678901.5,\"General\")", "12345678902"),
        ("=TEXT(12.123456789123,\"General\")", "12.12345679"),
        ("=TEXT(-1234567890.5,\"General\")", "-1234567891"),
        ("=TEXT(9999999999.5,\"General\")", "10000000000"),
        ("=TEXT(99999999999,\"General\")", "99999999999"),
        ("=TEXT(3.14159265358979,\"General\")", "3.141592654"),
        ("=TEXT(-3.14159265358979,\"General\")", "-3.141592654"),
        ("=TEXT(0.123456789012,\"General\")", "0.123456789"),
        ("=TEXT(0.000012345,\"General\")", "0.000012345"),
        ("=TEXT(0.9999999999,\"General\")", "1"),
        ("=TEXT(1234567890.5,\"General;General\")", "1234567891"),
    ]);
}

#[test]
fn text_general_shows_the_exponent_for_twelve_whole_digits() {
    assert_results(&[
        ("=TEXT(99999999999.5,\"General\")", "1E+11"),
        ("=TEXT(-99999999999.5,\"General\")", "-1E+11"),
        ("=TEXT(123456789012,\"General\")", "1.23457E+11"),
        ("=TEXT(-123456789012,\"General\")", "-1.23457E+11"),
        ("=TEXT(110000000000000,\"General\")", "1.1E+14"),
        ("=TEXT(110000000000000,\"General;h:mm\")", "1.1E+14"),
        ("=TEXT(1E+200,\"General\")", "1E+200"),
    ]);
}

#[test]
fn text_general_rounds_half_away_from_zero() {
    assert_results(&[
        ("=TEXT(123456500000,\"General\")", "1.23457E+11"),
        ("=TEXT(-123456500000,\"General\")", "-1.23457E+11"),
        ("=TEXT(2.5,\"General\")", "2.5"),
        ("=TEXT(1.0000000005,\"General\")", "1.000000001"),
    ]);
}

#[test]
fn text_general_moves_the_exponent_when_the_mantissa_rounds_to_ten() {
    assert_results(&[
        ("=TEXT(999999999999,\"General\")", "1E+12"),
        ("=TEXT(-999999999999,\"General\")", "-1E+12"),
        ("=TEXT(0.000000009999999,\"General\")", "1E-08"),
        ("=TEXT(0.000000001234567,\"General\")", "1.23457E-09"),
        ("=TEXT(0,\"General\")", "0"),
    ]);
}
