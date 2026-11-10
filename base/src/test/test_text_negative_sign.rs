#![allow(clippy::unwrap_used)]

// IronXL: TEXT shows `-` for a value that is below zero after the rounding, also when it rounds
// to one unit of its last digit, and no sign for a value that rounds to zero.

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
fn text_keeps_the_sign_of_a_value_that_rounds_to_one_unit_of_its_last_digit() {
    assert_results(&[
        ("=TEXT(-1,\"0\")", "-1"),
        ("=TEXT(-1.4,\"0\")", "-1"),
        ("=TEXT(-0.5,\"0\")", "-1"),
        ("=TEXT(-0.1,\"0.0\")", "-0.1"),
        ("=TEXT(-0.05,\"0.0\")", "-0.1"),
        ("=TEXT(-1,\"#,##0\")", "-1"),
        ("=TEXT(-0.01,\"0%\")", "-1%"),
        ("=TEXT(-1,\"\"\"x\"\"0\")", "-x1"),
    ]);
}

#[test]
fn text_shows_no_sign_for_a_value_that_rounds_to_zero() {
    assert_results(&[("=TEXT(-0.4,\"0\")", "0"), ("=TEXT(-0.04,\"0.0\")", "0.0")]);
}

#[test]
fn text_keeps_the_other_negative_texts() {
    assert_results(&[
        ("=TEXT(-2,\"0\")", "-2"),
        ("=TEXT(-1,\"0;(0)\")", "(1)"),
        ("=TEXT(-0.5,\".0\")", "-.5"),
        ("=TEXT(-1234,\"0E+0\")", "-1E+3"),
    ]);
}
