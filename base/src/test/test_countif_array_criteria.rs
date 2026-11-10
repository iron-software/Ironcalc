#![allow(clippy::unwrap_used)]

//! IronXL: COUNTIF with an array or a range of criteria gives one count for each criterion, in the
//! shape of the criteria, as SUMIF does and as in Excel. A scalar criterion gives one count.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!B1` text, `B2` 5, `E1` 5, and `E2` text. The other cells are empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("E1", "5");
    model._set("E2", "text");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model. Each formula gives
/// one value, so no result spills into the next formula.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("H{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("H{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn countif_gives_one_count_for_each_criterion_of_an_array() {
    assert_results(&[
        ("=SUMPRODUCT(COUNTIF(B1:B2,{5,6}))", "1"),
        ("=SUMPRODUCT(COUNTIF(B2:B2,{5,6}))", "1"),
        ("=SUMPRODUCT(COUNTIF(B2,{5,6}))", "1"),
        ("=SUM(COUNTIF(B1:B2,{5,6}))", "1"),
        ("=SUM(COUNTIF(B1:B2,{5,5}))", "2"),
    ]);
}

#[test]
fn each_criterion_follows_the_criteria_grammar() {
    assert_results(&[
        ("=SUM(COUNTIF(B1:B2,{5;\"text\"}))", "2"),
        ("=SUMPRODUCT(COUNTIF(B1:B2,E1:E2))", "2"),
        ("=SUM(COUNTIF(B1:B2,{\">0\",\"*\"}))", "2"),
        ("=SUM(COUNTIF(B1:B2,{\"<>text\",\"t?xt\"}))", "2"),
        ("=SUM(COUNTIF(B1:B2,{5,#N/A}))", "1"),
        ("=SUM(COUNTIF(B1:B2,{TRUE,5}))", "1"),
        // A whole column counts the empty cells outside the used range for each criterion.
        ("=SUM(COUNTIF(A:A,{\"\",5}))", "1048576"),
        ("=SUM(COUNTIF(B:B,{\"\",5}))", "1048575"),
    ]);
}

#[test]
fn the_result_has_the_shape_of_the_criteria() {
    assert_results(&[
        ("=INDEX(COUNTIF(B1:B2,{5,6}),1,1)", "1"),
        ("=INDEX(COUNTIF(B1:B2,{5,6}),1,2)", "0"),
        ("=INDEX(COUNTIF(B1:B2,{5;\"text\"}),2,1)", "1"),
        ("=INDEX(COUNTIF(B1:B2,E1:E2),2,1)", "1"),
        ("=INDEX(COUNTIF(B1:B2,{5,6;\"text\",7}),2,1)", "1"),
    ]);
}

#[test]
fn an_array_result_spills_from_the_formula_cell() {
    let mut model = model();
    model._set("H1", "=COUNTIF(B1:B2,{5,6})");
    model._set("H3", "=COUNTIF(B1:B2,E1:E2)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "1");
    assert_eq!(model._get_text("I1"), "0");
    assert_eq!(model._get_text("H3"), "1");
    assert_eq!(model._get_text("H4"), "1");
}

#[test]
fn an_error_of_the_criteria_range_gives_that_error() {
    assert_results(&[
        ("=SUM(COUNTIF(OFFSET(A1,-1,0),{5,6}))", "#REF!"),
        ("=COUNTIF(OFFSET(A1,-1,0),{5,6})", "#REF!"),
        ("=COUNTIF({5,6},{5,6})", "#VALUE!"),
        ("=COUNTIF(B1:B2)", "#ERROR!"),
    ]);
}

#[test]
fn a_scalar_criterion_and_the_other_criteria_functions_keep_their_results() {
    assert_results(&[
        ("=COUNTIF(B1:B2,5)", "1"),
        ("=COUNTIF(B1:B2,E1)", "1"),
        ("=COUNTIF(B1:B2,\"text\")", "1"),
        ("=COUNTIF(A:A,\"\")", "1048576"),
        ("=SUM(SUMIF(B1:B2,{5,6}))", "5"),
        ("=INDEX(SUMIF(B1:B2,{5,6}),1,2)", "0"),
        ("=COUNTIFS(B1:B2,5,B1:B2,\"<>x\")", "1"),
    ]);
}
