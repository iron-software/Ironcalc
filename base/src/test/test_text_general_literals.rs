#![allow(clippy::unwrap_used)]

// IronXL: TEXT with a section of General and literal parts gives the General text in the place
// of General, as Excel does: TEXT(5,"""x""General") gives x5.

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
fn text_general_with_a_literal_gives_the_excel_text() {
    assert_results(&[
        ("=TEXT(5,\"\"\"x\"\"General\")", "x5"),
        ("=TEXT(-5,\"General;-General\")", "-5"),
        ("=TEXT(-5,\"General;\"\"-\"\"General\")", "-5"),
        ("=TEXT(5,\"-General\")", "-5"),
        ("=TEXT(1234567890.5,\"\"\"x\"\"General\")", "x1234567891"),
    ]);
}

#[test]
fn text_general_shows_the_literals_before_and_after_general() {
    assert_results(&[
        ("=TEXT(2.5,\"General\"\" kg\"\"\")", "2.5 kg"),
        ("=TEXT(-2.5,\"General;\"\"minus \"\"General\")", "minus 2.5"),
        ("=TEXT(5,\"\\xGeneral\")", "x5"),
        ("=TEXT(0,\"General;General;\"\"z\"\"General\")", "z0"),
        ("=TEXT(5,\"General\"\"x\"\"\")", "5x"),
        ("=TEXT(5,\"(General)\")", "(5)"),
        ("=TEXT(5,\"$ General _)\")", "$ 5  "),
        ("=TEXT(5,\"*-General\")", " 5"),
        ("=TEXT(5,\"[Red]\"\"x\"\"General\")", "x5"),
        ("=TEXT(5,\"\"\"a\"\"General\"\"b\"\";0\")", "a5b"),
    ]);
}

#[test]
fn text_general_shows_the_currency_symbol_in_its_place() {
    assert_results(&[
        ("=TEXT(5,\"[$€]\"\"x\"\"General\")", "€x5"),
        ("=TEXT(5,\"[$€]General\")", "€5"),
        ("=TEXT(5,\"General[$€]\")", "5€"),
        ("=TEXT(5,\"\"\"a\"\"[$€]General\"\"b\"\"\")", "a€5b"),
        ("=TEXT(5,\"[$€-407]General\")", "€5"),
        ("=TEXT(5,\"[$£]General[$€]\")", "£5€"),
        ("=TEXT(-5,\"General;[$€]General\")", "€5"),
        ("=TEXT(-5,\"[$€]General\")", "€-5"),
    ]);
}

#[test]
fn text_general_with_one_section_keeps_the_sign_in_the_general_text() {
    assert_results(&[
        ("=TEXT(-5,\"\"\"x\"\"General\")", "x-5"),
        ("=TEXT(-2.5,\"General\"\" kg\"\"\")", "-2.5 kg"),
    ]);
}

#[test]
fn text_general_without_a_literal_keeps_its_results() {
    assert_results(&[
        ("=TEXT(5,\"General\")", "5"),
        ("=TEXT(-5,\"General;General\")", "5"),
        ("=TEXT(-5,\"0;-0\")", "-5"),
        ("=TEXT(110000000000000,\"General;h:mm\")", "1.1E+14"),
        ("=TEXT(5,\"[Red]General\")", "5"),
    ]);
}

#[test]
fn text_general_with_a_digit_or_a_date_part_gives_value_error() {
    assert_results(&[
        ("=TEXT(5,\"0General\")", "#VALUE!"),
        ("=TEXT(5,\"General0\")", "#VALUE!"),
        ("=TEXT(5,\"dd General\")", "#VALUE!"),
        ("=TEXT(5,\"General General\")", "#VALUE!"),
        ("=TEXT(5,\"@General\")", "#VALUE!"),
    ]);
}
