#![allow(clippy::unwrap_used)]

//! IronXL: the white space after a criteria operator does not count, as in Excel, and the public criteria grammar
//! and wildcard matcher agree with the formulas.

use crate::criteria::{CriteriaValue, Criterion, Wildcard};
use crate::expressions::token::Error;
use crate::formatter::dates::DateSystem;
use crate::locale::{get_locale, Locale};
use crate::model::Model;
use crate::test::util::new_empty_model;

fn en_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

/// D2:D6 holds 100, "n/a", an empty cell, 2500, and 1200.5. E2:E4 holds 31 December 2023, 2 January 2024, and
/// 1 June 2024.
fn amounts() -> Model<'static> {
    let mut model = new_empty_model();
    model._set("D2", "100");
    model._set("D3", "n/a");
    model._set("D5", "2500");
    model._set("D6", "1200.5");
    model._set("E2", "=DATE(2023,12,31)");
    model._set("E3", "=DATE(2024,1,2)");
    model._set("E4", "=DATE(2024,6,1)");
    model
}

/// The text of `formula` over the amounts.
fn result(formula: &str) -> String {
    let mut model = amounts();
    model._set("A1", formula);
    model.evaluate();
    model._get_text("A1")
}

#[test]
fn white_space_after_the_operator_compares_with_the_number() {
    // (criteria, SUMIF, COUNTIF, AVERAGEIF)
    let cases = [
        ("> 1000", "3700.5", "2", "1850.25"),
        (">=  2500", "2500", "1", "2500"),
        ("<> 100", "3700.5", "4", "1850.25"),
        ("= 100", "100", "1", "100"),
        ("<  0", "0", "0", "#DIV/0!"),
        ("<= 100", "100", "1", "100"),
    ];
    for (criteria, sum, count, average) in cases {
        assert_eq!(
            result(&format!("=SUMIF(D2:D6,\"{criteria}\")")),
            sum,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=COUNTIF(D2:D6,\"{criteria}\")")),
            count,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=AVERAGEIF(D2:D6,\"{criteria}\")")),
            average,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=SUMIFS(D2:D6,D2:D6,\"{criteria}\")")),
            sum,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=COUNTIFS(D2:D6,\"{criteria}\")")),
            count,
            "{criteria}"
        );
    }
}

#[test]
fn the_forms_without_white_space_keep_their_results() {
    let cases = [
        (">1000", "3700.5", "2", "1850.25"),
        (">=2500", "2500", "1", "2500"),
        ("<>100", "3700.5", "4", "1850.25"),
        ("=100", "100", "1", "100"),
        ("100", "100", "1", "100"),
        ("<0", "0", "0", "#DIV/0!"),
        ("<=100", "100", "1", "100"),
        ("n/a", "0", "1", "#DIV/0!"),
        ("n*", "0", "1", "#DIV/0!"),
    ];
    for (criteria, sum, count, average) in cases {
        assert_eq!(
            result(&format!("=SUMIF(D2:D6,\"{criteria}\")")),
            sum,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=COUNTIF(D2:D6,\"{criteria}\")")),
            count,
            "{criteria}"
        );
        assert_eq!(
            result(&format!("=AVERAGEIF(D2:D6,\"{criteria}\")")),
            average,
            "{criteria}"
        );
    }
}

#[test]
fn white_space_after_the_operator_compares_with_the_date() {
    assert_eq!(result("=COUNTIF(E2:E4,\"> 1/1/2024\")"), "2");
    assert_eq!(result("=COUNTIF(E2:E4,\">1/1/2024\")"), "2");
    assert_eq!(result("=COUNTIF(E2:E4,\"<= 1/2/2024\")"), "2");
    assert_eq!(result("=COUNTIFS(E2:E4,\"<> 1/2/2024\")"), "2");
}

#[test]
fn white_space_after_the_operator_of_a_text_does_not_count() {
    assert_eq!(result("=COUNTIF(D2:D6,\"= n/a\")"), "1");
    assert_eq!(
        result("=COUNTIF(D2:D6,\"<> n/a\")"),
        result("=COUNTIF(D2:D6,\"<>n/a\")")
    );
    assert_eq!(result("=COUNTIF(D2:D6,\"> m\")"), "1");
}

#[test]
fn a_text_without_an_operator_keeps_its_white_space() {
    assert_eq!(result("=COUNTIF(D2:D6,\" n/a\")"), "0");
}

#[test]
fn the_public_criterion_reads_a_number_criterion() {
    let criterion = Criterion::new("> 1000", en_locale(), DateSystem::Date1900);
    assert!(criterion.matches(&CriteriaValue::Number(1200.5)));
    assert!(!criterion.matches(&CriteriaValue::Number(100.0)));
    assert!(!criterion.matches(&CriteriaValue::Text("n/a")));
    assert!(!criterion.matches(&CriteriaValue::Empty));
    assert!(!criterion.matches(&CriteriaValue::Boolean(true)));
}

#[test]
fn the_public_criterion_reads_text_wildcards_errors_and_empty() {
    let en = en_locale();
    let text = Criterion::new("N/A", en, DateSystem::Date1900);
    assert!(text.matches(&CriteriaValue::Text("n/a")));
    assert!(!text.matches(&CriteriaValue::Text("n/a2")));
    let wildcard = Criterion::new("n*", en, DateSystem::Date1900);
    assert!(wildcard.matches(&CriteriaValue::Text("North")));
    assert!(!wildcard.matches(&CriteriaValue::Text("south")));
    let escaped = Criterion::new("~*", en, DateSystem::Date1900);
    assert!(escaped.matches(&CriteriaValue::Text("*")));
    assert!(!escaped.matches(&CriteriaValue::Text("x")));
    let error = Criterion::new("#DIV/0!", en, DateSystem::Date1900);
    assert!(error.matches(&CriteriaValue::Error(Error::DIV)));
    assert!(!error.matches(&CriteriaValue::Error(Error::NA)));
    let empty = Criterion::new("", en, DateSystem::Date1900);
    assert!(empty.matches(&CriteriaValue::Empty));
    assert!(!empty.matches(&CriteriaValue::Number(0.0)));
    let not_empty = Criterion::new("<>", en, DateSystem::Date1900);
    assert!(not_empty.matches(&CriteriaValue::Number(0.0)));
    assert!(!not_empty.matches(&CriteriaValue::Empty));
}

#[test]
fn the_public_criterion_reads_a_date_in_the_date_system() {
    let en = en_locale();
    // 2 January 2024 is the serial 45293 in the 1900 date system, and 43831 in the 1904 date system.
    let date1900 = Criterion::new("> 1/1/2024", en, DateSystem::Date1900);
    assert!(date1900.matches(&CriteriaValue::Number(45293.0)));
    assert!(!date1900.matches(&CriteriaValue::Number(43831.0)));
    let date1904 = Criterion::new("> 1/1/2024", en, DateSystem::Date1904);
    assert!(date1904.matches(&CriteriaValue::Number(43831.0)));
    assert!(!date1904.matches(&CriteriaValue::Number(43830.0)));
}

#[test]
fn the_public_wildcard_matches_as_excel_does() {
    let exact = Wildcard::new("?/A", true).unwrap();
    assert!(exact.is_match("n/a"));
    assert!(!exact.is_match("nn/a"));
    let part = Wildcard::new("b?", false).unwrap();
    assert!(part.is_match("abc"));
    let escaped = Wildcard::new("~?", true).unwrap();
    assert!(escaped.is_match("?"));
    assert!(!escaped.is_match("x"));
    let any = Wildcard::new("*", true).unwrap();
    assert!(any.is_match(""));
}
