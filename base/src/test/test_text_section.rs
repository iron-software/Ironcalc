#![allow(clippy::unwrap_used)]

// IronXL: TEXT shows a text that does not read as a number with the text section of the code,
// as Excel does. Without a text section, the text stays unchanged.

use crate::test::util::new_empty_model;

fn assert_results(cases: &[(&str, &str)]) {
    let mut model = new_empty_model();
    model._set("Z1", "abc");
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
fn text_shows_a_text_with_the_text_section() {
    assert_results(&[
        ("=TEXT(\"abc\",\"0;0;0;\"\"t:\"\"@\")", "t:abc"),
        ("=TEXT(\"abc\",\"0;0;0;[$-409]\"\"t:\"\"@\")", "t:abc"),
        ("=TEXT(\"abc\",\"\"\"t:\"\"@\")", "t:abc"),
        ("=TEXT(\"abc\",\"@\")", "abc"),
        ("=TEXT(Z1,\"0;0;0;\"\"t:\"\"@\")", "t:abc"),
    ]);
}

#[test]
fn text_shows_each_text_item_of_an_array_with_the_text_section() {
    assert_results(&[
        (
            "=INDEX(TEXT({\"abc\",\"1\"},\"0;0;0;\"\"t:\"\"@\"),1,1)",
            "t:abc",
        ),
        (
            "=INDEX(TEXT({\"abc\",\"1\"},\"0;0;0;\"\"t:\"\"@\"),1,2)",
            "1",
        ),
        ("=INDEX(TEXT({\"abc\"},\"0.00\"),1,1)", "abc"),
    ]);
}

#[test]
fn text_keeps_a_text_without_a_text_section() {
    assert_results(&[
        ("=TEXT(\"abc\",\"0\")", "abc"),
        ("=TEXT(\"abc\",\"0;0\")", "abc"),
        ("=TEXT(\"abc\",\"0;0;0;0@\")", "abc"),
        ("=TEXT(\"abc\",NA())", "abc"),
        ("=TEXT(\"12\",\"0;0;0;\"\"t:\"\"@\")", "12"),
        ("=TEXT(\"12\",\"0.00\")", "12.00"),
        ("=TEXT(\"2020-01-15\",\"m/d/yyyy\")", "1/15/2020"),
        ("=TEXT(\"1900-01-15\",\"m/d/yyyy\")", "1/15/1900"),
    ]);
}
