#![allow(clippy::unwrap_used)]

//! IronXL: `ROWS` and `COLUMNS` give the number of rows and columns of an array, as in Excel:
//! an array constant, a computed array, and a reference. A scalar is an array of one element,
//! and an error argument gives its error.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!L1:L4` 2, 1, 2, 1 and the workbook name `Rate` (0.2).
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    for (reference, input) in [("L1", "2"), ("L2", "1"), ("L3", "2"), ("L4", "1")] {
        model._set(reference, input);
    }
    for result in model.add_defined_names(&[("Rate", None, "0.2")]) {
        result.unwrap();
    }
    model
}

/// Checks that each formula gives its expected text, evaluated in one model.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("Z{}", index + 30), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("Z{}", index + 30)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn rows_and_columns_give_the_size_of_an_array_of_the_issue_table() {
    assert_results(&[
        ("=ROWS({1,2;3,4})", "2"),
        ("=COLUMNS({1,2,3})", "3"),
        ("=ROWS(L1:L4*1)", "4"),
        ("=ROWS(SUMIF(L1:L4,{1,2;3,4}))", "2"),
    ]);
}

#[test]
fn rows_and_columns_give_the_size_of_an_array_constant_and_a_computed_array() {
    assert_results(&[
        ("=COLUMNS({1,2;3,4})", "2"),
        ("=ROWS({1,2,3})", "1"),
        ("=ROWS({1;2;3})", "3"),
        ("=COLUMNS({1;2;3})", "1"),
        ("=COLUMNS(L1:L4*1)", "1"),
        ("=COLUMNS(SUMIF(L1:L4,{1,2;3,4}))", "2"),
        ("=ROWS(TRANSPOSE(L1:L4))", "1"),
        ("=COLUMNS(TRANSPOSE(L1:L4))", "4"),
        ("=ROWS(L1:L4>1)", "4"),
        ("=ROWS(-L1:L4)", "4"),
        ("=ROWS(L1:L4%)", "4"),
        ("=ROWS({1,#N/A;3,4})", "2"),
        ("=COLUMNS({1,#N/A;3,4})", "2"),
        ("=SUM(ROWS({1,2;3,4}))", "2"),
    ]);
}

#[test]
fn rows_and_columns_give_one_for_a_scalar() {
    assert_results(&[
        ("=ROWS(5)", "1"),
        ("=COLUMNS(5)", "1"),
        ("=ROWS(\"abc\")", "1"),
        ("=COLUMNS(TRUE)", "1"),
        ("=ROWS(Rate)", "1"),
        ("=COLUMNS(Rate)", "1"),
        ("=ISREF(Rate)", "FALSE"),
    ]);
}

#[test]
fn rows_and_columns_give_the_error_of_an_error_argument() {
    assert_results(&[
        ("=ROWS(1/0)", "#DIV/0!"),
        ("=COLUMNS(1/0)", "#DIV/0!"),
        ("=ROWS(#N/A)", "#N/A"),
        ("=ROWS(OFFSET(A1,-1,0))", "#REF!"),
        ("=ROWS(Nope)", "#NAME?"),
        ("=ROWS(LAMBDA(x,x))", "#VALUE!"),
        ("=COLUMNS(LAMBDA(x,x))", "#VALUE!"),
    ]);
}

#[test]
fn rows_and_columns_of_a_reference_keep_their_results() {
    assert_results(&[
        ("=ROWS(L1:L4)", "4"),
        ("=COLUMNS(L1:N4)", "3"),
        ("=ROWS(L1)", "1"),
        ("=COLUMNS(L1)", "1"),
        ("=ROWS(A:A)", "1048576"),
        ("=COLUMNS(1:1)", "16384"),
        ("=ROWS(INDEX(L1:L4,0,1))", "4"),
        ("=COLUMNS(INDEX(L1:N4,1,0))", "3"),
        ("=ROWS()", "#ERROR!"),
        ("=ROWS(L1:L4,L1)", "#ERROR!"),
        ("=COLUMNS()", "#ERROR!"),
        ("=COLUMNS(L1:L4,L1)", "#ERROR!"),
    ]);
}

#[test]
fn rows_of_a_computed_array_follow_a_change_of_its_input() {
    let mut model = model();
    model._set("Z30", "=ROWS(FILTER(L1:L4,L1:L4>1))");
    model.evaluate();
    assert_eq!(model._get_text("Z30"), "2");
    model._set("L2", "5");
    model.evaluate();
    assert_eq!(model._get_text("Z30"), "3");
}
