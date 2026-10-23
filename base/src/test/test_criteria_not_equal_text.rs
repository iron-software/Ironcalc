#![allow(clippy::unwrap_used)]

//! IronXL: a `"<>text"` criterion matches each value that is not equal to that text, as in Excel: numbers,
//! Booleans, errors, empty cells, and other text. The compare ignores case.

use crate::criteria::{CriteriaValue, Criterion};
use crate::expressions::token::Error;
use crate::formatter::dates::DateSystem;
use crate::locale::get_locale;
use crate::test::util::new_empty_model;

/// The text of `formula` over A1:A4 = 1, "y", an empty cell, and TRUE, B1:B15 = 1 to 15, and C1:C3 = "x",
/// #N/A, and 2.
fn result(formula: &str) -> String {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "y");
    model._set("A4", "TRUE");
    for row in 1..=15 {
        model._set(&format!("B{row}"), &row.to_string());
    }
    model._set("C1", "x");
    model._set("C2", "=NA()");
    model._set("C3", "2");
    model._set("Z1", formula);
    model.evaluate();
    model._get_text("Z1")
}

#[test]
fn a_not_equal_text_criterion_counts_numbers_booleans_and_empty_cells() {
    assert_eq!(result("=COUNTIF(A1:A4,\"<>x\")"), "4");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>X\")"), "4");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>y\")"), "3");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>Y\")"), "3");
    assert_eq!(result("=COUNTIF(A1:A4,\"<> y\")"), "3");
    assert_eq!(result("=COUNTIFS(A1:A4,\"<>x\")"), "4");
    assert_eq!(result("=COUNTIFS(A1:A4,\"<>y\",A1:A4,\"<>1\")"), "2");
}

#[test]
fn a_not_equal_text_criterion_adds_the_numbers_of_the_range() {
    assert_eq!(result("=SUMIF(A1:A4,\"<>x\")"), "1");
    assert_eq!(result("=AVERAGEIF(A1:A4,\"<>x\")"), "1");
    assert_eq!(result("=SUMIFS(A1:A4,A1:A4,\"<>x\")"), "1");
    assert_eq!(result("=AVERAGEIFS(A1:A4,A1:A4,\"<>x\")"), "1");
    assert_eq!(result("=SUMIF(B1:B15,\"<>x\")"), "120");
}

#[test]
fn a_not_equal_text_criterion_counts_an_error_cell() {
    assert_eq!(result("=COUNTIF(C1:C3,\"<>x\")"), "2");
    assert_eq!(result("=COUNTIF(C1:C3,\"<>y\")"), "3");
    assert_eq!(result("=SUMIF(C1:C3,\"<>x\")"), "#N/A");
}

#[test]
fn a_not_equal_text_criterion_counts_the_empty_cells_of_a_whole_column() {
    assert_eq!(result("=COUNTIF(B:B,\"<>x\")"), "1048576");
    assert_eq!(result("=COUNTIFS(B:B,\"<>x\")"), "1048576");
}

#[test]
fn the_other_not_equal_forms_keep_their_results() {
    assert_eq!(result("=COUNTIF(A1:A4,\"<>1\")"), "3");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>TRUE\")"), "3");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>#N/A\")"), "4");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>n*\")"), "4");
    assert_eq!(result("=COUNTIF(A1:A4,\"<>\")"), "3");
}

#[test]
fn the_public_criterion_matches_each_value_that_is_not_the_text() {
    let criterion = Criterion::new("<>x", get_locale("en").unwrap(), DateSystem::Date1900);
    assert!(criterion.matches(&CriteriaValue::Number(1.0)));
    assert!(criterion.matches(&CriteriaValue::Text("y")));
    assert!(criterion.matches(&CriteriaValue::Empty));
    assert!(criterion.matches(&CriteriaValue::Boolean(true)));
    assert!(criterion.matches(&CriteriaValue::Error(Error::NA)));
    assert!(!criterion.matches(&CriteriaValue::Text("x")));
    assert!(!criterion.matches(&CriteriaValue::Text("X")));
}
