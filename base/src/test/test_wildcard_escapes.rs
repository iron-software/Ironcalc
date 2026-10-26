#![allow(clippy::unwrap_used)]

//! IronXL: `~` makes the next character of a wildcard literal in every position, as in Excel.
//! An escaped `?` or `*` directly before `~~` keeps its meaning.

use crate::cell::CellValue;
use crate::criteria::Wildcard;
use crate::test::util::new_empty_model;

/// True when the exact wildcard `pattern` matches `text`.
fn matches(pattern: &str, text: &str) -> bool {
    Wildcard::new(pattern, true).unwrap().is_match(text)
}

#[test]
fn an_escaped_question_mark_before_an_escaped_tilde_is_literal() {
    assert!(matches("~?~~", "?~"));
    assert!(!matches("~?~~", "?\\"));
    assert!(!matches("~?~~", "x~"));
    assert!(matches("~?~~*", "?~abc"));
    assert!(matches("~?~?~~é", "??~É"));
}

#[test]
fn an_escaped_star_before_an_escaped_tilde_is_literal() {
    assert!(matches("~*~~", "*~"));
    assert!(!matches("~*~~", "ab~"));
    assert!(matches("~~~*", "~*"));
    assert!(matches("~~~?", "~?"));
    assert!(!matches("~~~?", "~x"));
}

#[test]
fn the_other_wildcard_rules_stay() {
    assert!(matches("a?c", "ABC"));
    assert!(!matches("a?c", "ac"));
    assert!(matches("a*", "a"));
    assert!(matches("~~*", "~x"));
    assert!(matches("~x", "x"));
    assert!(matches("a~", "a"));
    assert!(matches("a.b(c)[d]+", "a.b(c)[d]+"));
    assert!(!matches("a.b", "axb"));
    assert!(matches("??", "??"));
    assert!(matches("~", ""));
    assert!(Wildcard::new("b~?", false).unwrap().is_match("ab?c"));
    assert!(!Wildcard::new("b~?", false).unwrap().is_match("abxc"));
}

#[test]
fn countif_takes_an_escaped_question_mark_before_an_escaped_tilde() {
    let mut model = new_empty_model();
    model._set("A1", "?~");
    model._set("A2", "?\\");
    model._set("A3", "*~");
    model._set("A4", "~?");
    model._set("B1", "=COUNTIF(A1:A4,\"~?~~\")");
    model._set("B2", "=COUNTIF(A1:A4,\"~*~~\")");
    model._set("B3", "=COUNTIF(A1:A4,\"~~~?\")");
    model._set("B4", "=COUNTIF(A1:A4,\"<>~?~~\")");
    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B1"),
        Ok(CellValue::Number(1.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B2"),
        Ok(CellValue::Number(1.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B3"),
        Ok(CellValue::Number(1.0))
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!B4"),
        Ok(CellValue::Number(3.0))
    );
}
