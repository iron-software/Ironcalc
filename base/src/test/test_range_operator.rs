#![allow(clippy::unwrap_used)]

//! IronXL: the range operator `:` between two references gives the smallest range that holds
//! both sides, as in Excel. The order of the corners has no effect, and a side can be a range of
//! more than one cell. Two sides on different sheets, and a side that is not a reference, give
//! `#VALUE!`.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1` and `Sheet2`. `Sheet1!E1` is 1, `F1` is 2, `E2` is 3, `F2` is 4, `B1`
/// is text, `B2` is 5, `C1` is 7, and `C2` is 8. `Sheet2!E1` is 100.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model._set("E1", "1");
    model._set("F1", "2");
    model._set("E2", "3");
    model._set("F2", "4");
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("C1", "7");
    model._set("C2", "8");
    model._set("Sheet2!E1", "100");
    model
}

/// Checks that each formula, in column `H` of `Sheet1`, gives its expected text.
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
fn the_range_operator_orders_the_corners_of_a_computed_reference() {
    assert_results(&[
        ("=SUM(OFFSET(F2,0,0):E1)", "10"),
        ("=SUM(INDIRECT(\"F2\"):E1)", "10"),
        ("=SUM(OFFSET(B2,0,0):B1)", "5"),
        ("=SUM(INDEX(E1:F2,2,2):E1)", "10"),
        ("=ROWS(INDEX(B1:B2,2):B1)", "2"),
        ("=SUM(OFFSET(F1,0,0):E2)", "10"),
        ("=SUM(OFFSET(E2,0,0):F1)", "10"),
        ("=SUM(INDEX(E1:F2,2,2):INDEX(E1:F2,1,1))", "10"),
    ]);
}

#[test]
fn the_range_operator_holds_a_side_of_more_than_one_cell() {
    assert_results(&[
        ("=SUM(INDEX(E1:F2,0,1):F2)", "10"),
        ("=SUM(OFFSET(E1,0,0,2,1):F2)", "10"),
        ("=ROWS(OFFSET(E1,0,0,2,1):F1)", "2"),
        ("=COLUMNS(OFFSET(E1,0,0):OFFSET(F1,0,0,2,1))", "2"),
        ("=SUM(OFFSET(E1,0,0,1,2):C2)", "25"),
    ]);
}

#[test]
fn the_range_operator_keeps_the_results_that_match_excel() {
    assert_results(&[
        ("=SUM(F2:E1)", "10"),
        ("=SUM(E1:OFFSET(F2,0,0))", "10"),
        ("=SUM(OFFSET(E1,0,0):OFFSET(E1,0,0))", "1"),
        ("=SUM(B1:INDEX(B1:B2,2))", "5"),
        (
            "=SUM(INDIRECT(\"Sheet2!E1\"):INDIRECT(\"Sheet2!F2\"))",
            "100",
        ),
        ("=SUM(INDIRECT(\"Sheet2!E1\"):E1)", "#VALUE!"),
        ("=SUM(E1:INDIRECT(\"Sheet2!E1\"))", "#VALUE!"),
        ("=SUM(INDEX({1,2},1):E1)", "#VALUE!"),
        ("=SUM(E1:INDEX({1,2},1))", "#VALUE!"),
    ]);
}

#[test]
fn the_range_operator_follows_a_change_of_its_inputs() {
    let mut model = model();
    model._set("H1", "=SUM(OFFSET(F2,0,0):E1)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "10");
    model._set("E1", "11");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "20");
}
