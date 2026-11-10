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
fn point_before_the_first_digit_is_the_decimal_point() {
    assert_texts(&[
        (0.5, ".0", ".5"),
        (1.5, ".0", "1.5"),
        (-0.5, ".0", "-.5"),
        (0.05, ".0", ".1"),
        (0.96, ".0", "1.0"),
        (12.34, ".00", "12.34"),
        (1234.5, ".0", "1234.5"),
        (0.0, ".0", ".0"),
        (0.5, ".#", ".5"),
        (0.5, ".?", ".5"),
        (-0.5, ".0;.0", ".5"),
        (0.5, "\"x\".0", "x.5"),
        (-0.5, "\"x\".0", "-x.5"),
        (0.5, ".0%", "50.0%"),
        (0.5, "$.00", "$.50"),
    ]);
}

#[test]
fn point_without_digits_after_it_stays() {
    assert_texts(&[
        (5.0, "0.", "5."),
        (5.0, "#.", "5."),
        (-5.0, "0.", "-5."),
        (0.0, "#.", "."),
        (0.0, "0.", "0."),
        (5.5, "0.", "6."),
        (1234.0, "#,##0.", "1,234."),
        (5.0, "0.\"x\"", "5.x"),
        (0.5, "0.%", "50.%"),
        (-5.0, "0.;(0.)", "(5.)"),
    ]);
}

#[test]
fn point_without_digits_is_text() {
    assert_texts(&[(5.0, "\"a\".", "a.")]);
}

#[test]
fn scientific_exponent_is_a_multiple_of_the_whole_digits() {
    assert_texts(&[
        (1234.0, ".0E+0", ".1E+4"),
        (1234.0, ".00E+00", ".12E+04"),
        (0.5, ".0E+0", ".5E+0"),
        (0.96, ".0E+0", ".1E+1"),
        (-0.5, ".0E+0", "-.5E+0"),
        (0.0, ".0E+0", ".0E+0"),
        (5.0, "0.E+0", "5.E+0"),
        (1.5, "0.00E+00", "1.50E+00"),
        (12345.0, "##0.0E+0", "12.3E+3"),
        (999_999.0, "##0.0E+0", "1.0E+6"),
        (0.012345, "##0.0E+0", "12.3E-3"),
        (0.05, "##0.0E+0", "50.0E-3"),
        (9.999, "0.00E+00", "1.00E+01"),
        (99_600.0, "0.0E+0", "1.0E+5"),
        (0.05, "0.0E+0", "5.0E-2"),
        (0.0, "0.00E+00", "0.00E+00"),
        (-1234.0, "0.0E+0", "-1.2E+3"),
        (1.5, "0.0E-0", "1.5E0"),
        (0.15, "0.0E-0", "1.5E-1"),
    ]);
}

// IronXL: a negative value keeps its sign when its mantissa rounds to one unit of the last digit,
// as Excel shows -1E+3 for -1234 with 0E+0.
#[test]
fn scientific_mantissa_keeps_the_sign_of_the_value() {
    assert_texts(&[
        (-1234.0, "0E+0", "-1E+3"),
        (-12.0, "0E+0", "-1E+1"),
        (-1.2, "0E+0", "-1E+0"),
        (-0.0012, "0E+0", "-1E-3"),
        (-1100.0, "0.E+0", "-1.E+3"),
        (-1234.0, ".0E+0", "-.1E+4"),
        (-0.1, ".0E+0", "-.1E+0"),
        (-1234.0, ".00E+00", "-.12E+04"),
        (-1001.0, "0.00E+00", "-1.00E+03"),
        (-0.0, "0E+0", "0E+0"),
    ]);
}

#[test]
fn codes_with_digits_on_each_side_of_the_point_do_not_change() {
    assert_texts(&[
        (5.0, "0.0", "5.0"),
        (5.0, "#.#", "5."),
        (5.0, "0", "5"),
        (1234.5, "#,##0.00", "1,234.50"),
        (0.5, "0%", "50%"),
        (0.96, "0.0", "1.0"),
    ]);
}

#[test]
fn rounding_reads_fifteen_significant_digits() {
    assert_texts(&[
        (1.005, "0.00", "1.01"),
        (-1.005, "0.00", "-1.01"),
        (0.045, "0.0", "0.0"),
        (0.25, "0.0", "0.3"),
        (2.5, "0", "3"),
        (1.0 / 3.0, "0.0000000000000000000", "0.3333333333333330000"),
        (123_456_789_012_345_680.0, "0", "123456789012345680"),
    ]);
}
