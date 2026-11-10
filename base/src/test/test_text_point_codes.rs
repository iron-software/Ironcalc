#![allow(clippy::unwrap_used)]

// IronXL: TEXT shows a code with no digit placeholder before or after the point as Excel does,
// and a scientific code with the exponent of Excel.

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
fn text_shows_a_point_before_the_first_digit_as_the_decimal_point() {
    assert_results(&[
        ("=TEXT(0.5,\".0\")", ".5"),
        ("=TEXT(-0.5,\".0\")", "-.5"),
        ("=TEXT(1.5,\".0\")", "1.5"),
        ("=INDEX(TEXT({0.5,1.5},\".0\"),1,1)", ".5"),
        ("=INDEX(TEXT({0.5,1.5},\".0\"),1,2)", "1.5"),
    ]);
}

#[test]
fn text_keeps_a_point_without_digits_after_it() {
    assert_results(&[
        ("=TEXT(5,\"0.\")", "5."),
        ("=TEXT(0,\"#.\")", "."),
        ("=TEXT(-5,\"0.;(0.)\")", "(5.)"),
    ]);
}

#[test]
fn text_shows_the_exponent_of_excel() {
    assert_results(&[
        ("=TEXT(1234,\".0E+0\")", ".1E+4"),
        ("=TEXT(5,\"0.E+0\")", "5.E+0"),
        ("=TEXT(12345,\"##0.0E+0\")", "12.3E+3"),
        ("=TEXT(1.5,\"0.00E+00\")", "1.50E+00"),
    ]);
}
