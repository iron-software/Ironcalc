#![allow(clippy::unwrap_used)]

// IronXL: a number does not use a section with `@`, nor the fourth section, as Excel does. The
// number sections that are left pick the section by the usual rules.

use crate::{
    formatter::{
        dates::DateSystem,
        format::{format_number, format_text_number},
    },
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// Each case is a value, a code, and the text that Excel shows. The cell path and the `TEXT` path
// give the same text.
fn assert_texts(cases: &[(f64, &str, &str)]) {
    let locale = get_default_locale();
    for &(value, code, expected) in cases {
        let cell = format_number(value, code, locale);
        assert_eq!(cell.text, expected, "{value} with {code}");
        assert_eq!(cell.error, None, "{value} with {code}");
        let text = format_text_number(value, code, locale, DateSystem::Date1900);
        assert_eq!(text.text, expected, "TEXT of {value} with {code}");
        assert_eq!(text.error, None, "TEXT of {value} with {code}");
    }
}

#[test]
fn the_only_number_section_before_a_text_section_keeps_the_sign() {
    assert_texts(&[
        (-1.5, "0.00;@", "-1.50"),
        (-1234.5, "#,##0.00;@", "-1,234.50"),
        (-0.5, "0;@", "-1"),
        (-0.4, "0;@", "0"),
        (1.5, "0.00;@", "1.50"),
        (0.0, "0.00;@", "0.00"),
        (-1.5, "\"x\"0.00;@", "-x1.50"),
        (-5.0, "0.00;@;@;@", "-5.00"),
    ]);
}

#[test]
fn two_number_sections_before_a_text_section_pick_by_the_sign() {
    assert_texts(&[
        (0.0, "0.00;0.0;@", "0.00"),
        (1.5, "0.00;0.0;@", "1.50"),
        (-1.5, "0.00;0.0;@", "1.5"),
        (-1.5, "0.00;(0.00);@;@", "(1.50)"),
    ]);
}

#[test]
fn codes_without_a_number_section_before_the_text_section_keep_their_texts() {
    assert_texts(&[
        (0.5, "@", "0.5"),
        (-0.5, "@", "-0.5"),
        (-0.5, "@;@", "0.5"),
        (-1.5, "0.00;(0.00);0;@", "(1.50)"),
        (0.0, "0;-0;;@", ""),
        (2.0, "0;-0;;@", "2"),
        (-2.0, "0;-0;;@", "-2"),
        (-1.5, "0.00;0.0", "1.5"),
        (0.0, "0;0;\"zero\"", "zero"),
    ]);
}

#[test]
fn a_date_section_before_a_text_section_serves_every_number() {
    let locale = get_default_locale();
    assert_eq!(format_number(1.0, "m/d/yyyy;@", locale).text, "1/1/1900");
    let negative = format_text_number(-1.0, "m/d/yyyy;@", locale, DateSystem::Date1900);
    assert_eq!(negative.text, "#VALUE!");
}

#[test]
fn more_than_four_sections_give_an_error() {
    let locale = get_default_locale();
    let formatted = format_text_number(12.0, "0;0;0;@;0", locale, DateSystem::Date1900);
    assert_eq!(formatted.text, "#VALUE!");
    assert_eq!(formatted.error.as_deref(), Some("Too many parts"));
}
