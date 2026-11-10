#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// IronXL: each case is a value, a code, and the text that Excel shows.
fn assert_texts(cases: &[(f64, &str, &str)]) {
    let locale = get_default_locale();
    for &(value, code, expected) in cases {
        let formatted = format_number(value, code, locale);
        assert_eq!(formatted.text, expected, "{value} with {code}");
        assert_eq!(formatted.error, None, "{value} with {code}");
    }
}

#[test]
fn negative_value_that_rounds_to_one_unit_of_its_last_digit_keeps_its_sign() {
    assert_texts(&[
        (-1.0, "0", "-1"),
        (-1.4, "0", "-1"),
        (-0.5, "0", "-1"),
        (-0.1, "0.0", "-0.1"),
        (-0.05, "0.0", "-0.1"),
        (-1.0, "#,##0", "-1"),
        (-0.01, "0%", "-1%"),
        (-1.0, "\"x\"0", "-x1"),
        (-1.0, "#", "-1"),
    ]);
}

#[test]
fn negative_value_that_rounds_to_zero_shows_no_sign() {
    assert_texts(&[
        (-0.4, "0", "0"),
        (-0.04, "0.0", "0.0"),
        (-0.004, "0%", "0%"),
    ]);
}

#[test]
fn other_negative_values_keep_their_texts() {
    assert_texts(&[
        (-2.0, "0", "-2"),
        (-1.0, "0;(0)", "(1)"),
        (-0.5, ".0", "-.5"),
        (-1234.0, "0E+0", "-1E+3"),
        (-1.0, "0%", "-100%"),
        (-1234.5, "#,##0.0", "-1,234.5"),
    ]);
}
