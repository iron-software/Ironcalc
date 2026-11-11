#![allow(clippy::unwrap_used)]

// IronXL: TEXT shows each digit of an exponent once, also when the exponent has more digits
// than the exponent placeholders of the code, as Excel does.

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
fn text_shows_each_digit_of_a_long_exponent_once() {
    assert_results(&[
        ("=TEXT(1E+100,\"0.00E+00\")", "1.00E+100"),
        ("=TEXT(-1.5E+200,\"0.00E+00\")", "-1.50E+200"),
        ("=TEXT(1.5E-100,\"0.00E+00\")", "1.50E-100"),
        ("=TEXT(9.999E+99,\"0.00E+00\")", "1.00E+100"),
    ]);
}

#[test]
fn text_shows_a_long_exponent_in_each_exponent_code_form() {
    assert_results(&[
        ("=TEXT(1E+100,\"0.00E-00\")", "1.00E100"),
        ("=TEXT(1E+102,\"##0.0E+00\")", "1.0E+102"),
        ("=TEXT(1E+100,\"0.00E+##\")", "1.00E+100"),
        ("=TEXT(-1E+100,\"0.00E+00;(0.00E+00)\")", "(1.00E+100)"),
    ]);
}

#[test]
fn text_keeps_an_exponent_that_fits_the_placeholders() {
    assert_results(&[
        ("=TEXT(1E+100,\"0E+0\")", "1E+100"),
        ("=TEXT(1E+99,\"0.00E+00\")", "1.00E+99"),
        ("=TEXT(1E+100,\"0.0E+0\")", "1.0E+100"),
        ("=TEXT(1E+100,\"0.00E+000\")", "1.00E+100"),
        ("=TEXT(1E+300,\"0.00E+0000\")", "1.00E+0300"),
        ("=TEXT(1E+5,\"0.00E+##\")", "1.00E+5"),
    ]);
}
