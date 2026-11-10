#![allow(clippy::unwrap_used)]

// IronXL: TEXT shows a number with an empty code as the empty text, as Excel does. A code with
// only locale tags without a currency symbol, such as `[$-409]`, is the empty code after the tags
// go.

use crate::test::util::new_empty_model;

fn assert_results(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    model._set("C1", "1");
    model._set("C2", "2");
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
fn text_shows_a_number_with_an_empty_code_as_the_empty_text() {
    assert_results(&[
        ("=TEXT(12,\"\")&\"|\"", "|"),
        ("=LEN(TEXT(12,\"\"))", "0"),
        ("=LEN(TEXT(-12,\"\"))", "0"),
        ("=LEN(TEXT(0,\"\"))", "0"),
        ("=LEN(TEXT(B1,\"\"))", "0"),
        ("=LEN(TEXT(\"12\",\"\"))", "0"),
    ]);
}

#[test]
fn text_shows_a_number_with_only_locale_tags_as_the_empty_text() {
    assert_results(&[
        ("=LEN(TEXT(12,\"[$-409]\"))", "0"),
        ("=LEN(TEXT(12,\"[$-409][$-409]\"))", "0"),
        ("=LEN(TEXT(12,\"[$-en-US]\"))", "0"),
        ("=LEN(TEXT(\"12\",\"[$-409]\"))", "0"),
    ]);
}

#[test]
fn text_shows_each_number_item_of_an_array_with_an_empty_code_as_the_empty_text() {
    assert_results(&[
        ("=LEN(INDEX(TEXT({1,2},\"\"),1,1))", "0"),
        ("=LEN(INDEX(TEXT({1,2},\"\"),1,2))", "0"),
        ("=LEN(INDEX(TEXT({1,2},\"[$-409]\"),1,1))", "0"),
        ("=LEN(INDEX(TEXT({1,2},\"[$-409]\"),1,2))", "0"),
        ("=LEN(INDEX(TEXT(C1:C2,\"\"),1,1))", "0"),
        ("=LEN(INDEX(TEXT(C1:C2,\"\"),2,1))", "0"),
    ]);
}

#[test]
fn text_keeps_the_other_results_of_an_empty_code() {
    assert_results(&[
        ("=TEXT(\"abc\",\"\")", "abc"),
        ("=TEXT(\"abc\",\"[$-409]\")", "abc"),
        ("=TEXT(TRUE,\"\")", "TRUE"),
        ("=TEXT(12,NA())", "#N/A"),
        ("=TEXT(12,\"[$\u{20ac}-409]\")", "\u{20ac}"),
        ("=TEXT(12,\";;;\")&\"|\"", "|"),
        ("=TEXT(0.5,\"ss[$-409].0\")", "#VALUE!"),
        ("=TEXT(12,\"[$-409\")", "#VALUE!"),
        ("=TEXT(0.5,\"[$-F400]h:mm\")", "12:00:00 PM"),
        ("=TEXT(12,\"0;0;0;@;0\")", "#VALUE!"),
    ]);
}
