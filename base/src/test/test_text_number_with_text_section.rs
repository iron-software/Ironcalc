#![allow(clippy::unwrap_used)]

// IronXL: TEXT of a number does not use a section with `@`, nor the fourth section, as Excel
// does. So `=TEXT(-1.5,"0.00;@")` gives `-1.50`.

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
fn text_of_a_number_skips_the_text_section() {
    assert_results(&[
        ("=TEXT(-1.5,\"0.00;@\")", "-1.50"),
        ("=TEXT(-1234.5,\"#,##0.00;@\")", "-1,234.50"),
        ("=TEXT(-0.5,\"0;@\")", "-1"),
        ("=TEXT(-0.4,\"0;@\")", "0"),
        ("=TEXT(1.5,\"0.00;@\")", "1.50"),
        ("=TEXT(0,\"0.00;@\")", "0.00"),
        ("=TEXT(0,\"0.00;0.0;@\")", "0.00"),
        ("=TEXT(-1.5,\"0.00;0.0;@\")", "1.5"),
    ]);
}

#[test]
fn text_of_other_codes_with_a_text_section_does_not_change() {
    assert_results(&[
        ("=TEXT(0.5,\"@\")", "0.5"),
        ("=TEXT(-0.5,\"@\")", "-0.5"),
        ("=TEXT(-1.5,\"0.00;(0.00);0;@\")", "(1.50)"),
        ("=TEXT(0,\"0;-0;;@\")&\"|\"", "|"),
        ("=TEXT(\"abc\",\"0;0;0;\"\"t:\"\"@\")", "t:abc"),
        ("=TEXT(12,\"0;0;0;@;0\")", "#VALUE!"),
    ]);
}
