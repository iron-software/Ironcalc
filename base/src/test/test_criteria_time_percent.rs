#![allow(clippy::unwrap_used)]

//! IronXL: a time, a date and time, and a percent criteria text compare as their numbers, as a date text does in
//! Excel. A text that the parsers refuse keeps its match as literal text.

use crate::criteria::{CriteriaValue, Criterion};
use crate::formatter::dates::DateSystem;
use crate::locale::get_locale;
use crate::test::util::new_empty_model;

/// The text of `formula` when A1 holds 43832 (2 January 2020), A2 and A3 hold 0.5, and B1:B3 hold `texts`.
fn result_with_texts(formula: &str, texts: [&str; 3]) -> String {
    let mut model = new_empty_model();
    model._set("A1", "43832");
    model._set("A2", "0.5");
    model._set("A3", "0.5");
    model._set("B1", texts[0]);
    model._set("B2", texts[1]);
    model._set("B3", texts[2]);
    model._set("C1", formula);
    model.evaluate();
    model._get_text("C1")
}

fn result(formula: &str) -> String {
    result_with_texts(formula, ["", "", ""])
}

fn assert_results(cases: &[(&str, &str)]) {
    for (formula, expected) in cases {
        assert_eq!(result(formula), *expected, "{formula}");
    }
}

#[test]
fn a_time_text_compares_as_its_fraction_of_a_day() {
    assert_results(&[
        ("=COUNTIF(A1:A3,\"12:00\")", "2"),
        ("=COUNTIF(A1:A3,\"=12:00\")", "2"),
        ("=COUNTIF(A1:A3,\">=12:00\")", "3"),
        ("=COUNTIF(A1:A3,\">12:00\")", "1"),
        ("=COUNTIF(A1:A3,\"<12:00\")", "0"),
        ("=COUNTIF(A1:A3,\"<=12:00\")", "2"),
        ("=COUNTIF(A1:A3,\"<>12:00\")", "1"),
        ("=COUNTIF(A1:A3,\"12:00 PM\")", "2"),
        ("=COUNTIF(A1:A3,\"12:00:00\")", "2"),
        ("=COUNTIF(A1:A3,\">= 12:00\")", "3"),
        ("=COUNTIF(A1:A3,\"<  12:00\")", "0"),
        ("=SUMIF(A1:A3,\"12:00\")", "1"),
        ("=COUNTIFS(A1:A3,\"12:00\")", "2"),
        ("=SUMIFS(A1:A3,A1:A3,\">=12:00\")", "43833"),
        ("=AVERAGEIF(A1:A3,\"12:00\")", "0.5"),
    ]);
}

#[test]
fn a_percent_text_compares_as_the_number_divided_by_100() {
    assert_results(&[
        ("=COUNTIF(A1:A3,\"50%\")", "2"),
        ("=COUNTIF(A1:A3,\"=50%\")", "2"),
        ("=COUNTIF(A1:A3,\">=50%\")", "3"),
        ("=COUNTIF(A1:A3,\">50%\")", "1"),
        ("=COUNTIF(A1:A3,\"<50%\")", "0"),
        ("=COUNTIF(A1:A3,\"<=50%\")", "2"),
        ("=COUNTIF(A1:A3,\"<>50%\")", "1"),
        ("=COUNTIF(A1:A3,\"50.0 %\")", "2"),
        ("=SUMIF(A1:A3,\"50%\")", "1"),
        ("=AVERAGEIF(A1:A3,\"50%\")", "0.5"),
    ]);
}

#[test]
fn a_date_and_time_text_compares_as_its_serial_number() {
    assert_results(&[
        ("=COUNTIF(A1:A3,\"1/2/2020 0:00\")", "1"),
        ("=COUNTIF(A1:A3,\"<1/2/2020 12:00\")", "3"),
        ("=COUNTIF(A1:A3,\">1/1/2020 23:59\")", "1"),
    ]);
}

#[test]
fn the_other_criteria_keep_their_results() {
    assert_results(&[
        ("=COUNTIF(A1:A3,\"1/2/2020\")", "1"),
        ("=COUNTIF(A1:A3,\"0.5\")", "2"),
        ("=COUNTIF(A1:A3,\">0.5\")", "1"),
        ("=COUNTIF(A1:A3,\"12:*\")", "0"),
        ("=COUNTIF(A1:A3,\"<>\")", "3"),
        ("=COUNTIF(A1:A3,\"\")", "0"),
        ("=COUNTIF(A1:A3,\"TRUE\")", "0"),
        ("=COUNTIF(A1:A3,\"#N/A\")", "0"),
    ]);
    let mut model = new_empty_model();
    model._set("A1", "1000");
    model._set("C1", "=COUNTIF(A1,\"$1,000\")");
    model._set("C2", "=VALUE(\"12:00\")");
    model._set("C3", "=VALUE(\"50%\")");
    model._set("C4", "=TIMEVALUE(\"12:00 PM\")");
    model.evaluate();
    assert_eq!(model._get_text("C1"), "0");
    assert_eq!(model._get_text("C2"), "0.5");
    assert_eq!(model._get_text("C3"), "0.5");
    assert_eq!(model._get_text("C4"), "0.5");
}

#[test]
fn a_text_that_the_parsers_refuse_matches_as_literal_text() {
    let texts = ["25:00", "12:60", "abc%"];
    for (criteria, expected) in [
        ("25:00", "1"),
        ("12:60", "1"),
        ("abc%", "1"),
        ("12:00:00.2.5", "0"),
        ("<>25:00", "2"),
        ("%", "0"),
    ] {
        let formula = format!("=COUNTIF(B1:B3,\"{criteria}\")");
        assert_eq!(result_with_texts(&formula, texts), expected, "{criteria}");
    }
    // A time text compares with a number only, never with a text cell.
    assert_eq!(
        result_with_texts("=COUNTIF(B1:B3,\"12:00\")", ["12:00x", "noon", ""]),
        "0"
    );
}

#[test]
fn the_public_criterion_reads_a_time_and_a_percent() {
    let en = get_locale("en").unwrap();
    let time = Criterion::new(">=12:00", en, DateSystem::Date1900);
    assert!(time.matches(&CriteriaValue::Number(0.5)));
    assert!(time.matches(&CriteriaValue::Number(43832.0)));
    assert!(!time.matches(&CriteriaValue::Number(0.25)));
    assert!(!time.matches(&CriteriaValue::Text("12:00")));
    let percent = Criterion::new("50%", en, DateSystem::Date1900);
    assert!(percent.matches(&CriteriaValue::Number(0.5)));
    assert!(!percent.matches(&CriteriaValue::Number(50.0)));
    let literal = Criterion::new("abc%", en, DateSystem::Date1900);
    assert!(literal.matches(&CriteriaValue::Text("ABC%")));
    assert!(!literal.matches(&CriteriaValue::Number(0.5)));
}

#[test]
fn the_public_criterion_reads_a_date_and_time_in_the_date_system() {
    let en = get_locale("en").unwrap();
    // 2 January 2020 is the serial 43832 in the 1900 date system, and 42370 in the 1904 date system.
    let date1900 = Criterion::new("1/2/2020 12:00", en, DateSystem::Date1900);
    assert!(date1900.matches(&CriteriaValue::Number(43832.5)));
    assert!(!date1900.matches(&CriteriaValue::Number(42370.5)));
    let date1904 = Criterion::new("1/2/2020 12:00", en, DateSystem::Date1904);
    assert!(date1904.matches(&CriteriaValue::Number(42370.5)));
    assert!(!date1904.matches(&CriteriaValue::Number(43832.5)));
}
