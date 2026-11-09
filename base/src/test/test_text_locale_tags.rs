#![allow(clippy::unwrap_used)]

// IronXL: TEXT accepts a locale tag without a currency symbol, such as `[$-409]`, as Excel does.
// A system tag, `[$-F800]` or `[$-F400]`, gives the en-US long date or long time.

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
fn text_removes_a_locale_tag_without_a_currency_symbol() {
    assert_results(&[
        ("=TEXT(0.5,\"[$-409]h:mm:ss\")", "12:00:00"),
        ("=TEXT(1234.5,\"[$-409]0.00\")", "1234.50"),
        ("=TEXT(1234.25,\"[$-en-US]0.00\")", "1234.25"),
        ("=TEXT(-1.25,\"0.00;[$-409]-0.00\")", "-1.25"),
        ("=TEXT(\"0.5\",\"[$-409]h:mm\")", "12:00"),
    ]);
}

#[test]
fn text_gives_the_en_us_code_of_a_system_tag() {
    assert_results(&[
        ("=TEXT(0.5,\"[$-F400]h:mm:ss AM/PM\")", "12:00:00 PM"),
        ("=TEXT(0.5,\"[$-f400]h:mm\")", "12:00:00 PM"),
        ("=TEXT(0.5,\"[$-x-systime]h:mm\")", "12:00:00 PM"),
        (
            "=TEXT(45000.5,\"[$-F800]dddd, mmmm dd, yyyy\")",
            "Wednesday, March 15, 2023",
        ),
        (
            "=TEXT(45000,\"[$-x-sysdate]d\")",
            "Wednesday, March 15, 2023",
        ),
    ]);
}

#[test]
fn text_removes_the_tag_from_each_item_of_an_array() {
    assert_results(&[
        ("=INDEX(TEXT({0.5,0.25},\"[$-409]h:mm\"),1,1)", "12:00"),
        ("=INDEX(TEXT({0.5,0.25},\"[$-409]h:mm\"),1,2)", "6:00"),
        ("=INDEX(TEXT({0.5,-0.25},\"[$-409]h:mm\"),1,2)", "#VALUE!"),
    ]);
}

#[test]
fn text_keeps_a_currency_tag_and_a_quoted_tag() {
    assert_results(&[
        (
            "=TEXT(1234.5678,\"[$\u{20ac}-409]0.00\")",
            "\u{20ac}1234.57",
        ),
        ("=TEXT(1234.25,\"[$$-409]0.00\")", "$1234.25"),
        ("=TEXT(12,\"\"\"[$-409]\"\"0\")", "[$-409]12"),
    ]);
}

#[test]
fn text_refuses_a_code_that_the_tag_does_not_fix() {
    assert_results(&[
        ("=TEXT(0.5,\"ss[$-409].0\")", "#VALUE!"),
        ("=TEXT(0.5,\"[$-409h:mm\")", "#VALUE!"),
        ("=TEXT(-0.25,\"[$-409]h:mm\")", "#VALUE!"),
    ]);
}
