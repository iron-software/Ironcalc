#![allow(clippy::unwrap_used)]

//! IronXL: the sum range of SUMIF and the average range of AVERAGEIF start at their top-left cell
//! and take the rows and columns of the criteria range, as in Excel. A range of another size gives
//! no error. The size rule of the IFS functions stays.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `B1` text, `B2` 5, `C1` 1, `C2` 10, `C3` 100, and `D1` 1000 on `Sheet1`. The
/// other cells are empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("C1", "1");
    model._set("C2", "10");
    model._set("C3", "100");
    model._set("D1", "1000");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model. Each formula is in
/// its own row of column `H`, so a spill goes into column `I`.
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
fn the_issue_rows_give_the_excel_results() {
    assert_results(&[
        ("=SUMIF(B1:B2,5,B1)", "5"),
        ("=AVERAGEIF(B1:B2,5,B1)", "5"),
        ("=SUMIF(B1:B2,5,B1:B5)", "5"),
        ("=AVERAGEIF(B1:B2,5,B1:B5)", "5"),
    ]);
}

#[test]
fn a_smaller_a_wider_and_a_larger_range_take_the_size_of_the_criteria_range() {
    assert_results(&[
        ("=SUMIF(B1:B2,5,C1)", "10"),
        ("=SUMIF(B1:B2,5,C1:D1)", "10"),
        ("=AVERAGEIF(B1:B2,5,C1)", "10"),
        ("=AVERAGEIF(B1:B2,5,C1:D1)", "10"),
        // A larger range reads no cell past the size of the criteria range.
        ("=SUMIF(B1:B2,\"\",C1:C5)", "0"),
        ("=AVERAGEIF(B1:B2,\"\",C1:C5)", "#DIV/0!"),
        ("=SUMIF(B1:B2,\"text\",C1:D5)", "1"),
        // A criteria range of more than one column.
        ("=SUMIF(B1:C1,\"text\",C3)", "100"),
        ("=SUMIF(B1:C1,1,C3)", "0"),
        ("=AVERAGEIF(B1:C1,\"<>text\",B3)", "100"),
        ("=AVERAGEIF(B1:C1,\"x\",B3)", "#DIV/0!"),
    ]);
}

#[test]
fn array_criteria_resize_the_sum_range() {
    let mut model = model();
    model._set("H1", "=SUMIF(B1:B2,{5,\"text\"},C1)");
    model._set("H2", "=SUM(SUMIF(B1:B2,{5,\"text\"},C1))");
    model._set("H3", "=SUM(SUMIF(B1:B2,{\"\"},C1:C5))");
    model._set("H4", "=SUM(SUMIF(B1:B2,B1:B2,C1:D1))");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "10");
    assert_eq!(model._get_text("I1"), "1");
    assert_eq!(model._get_text("H2"), "11");
    assert_eq!(model._get_text("H3"), "0");
    assert_eq!(model._get_text("H4"), "11");
}

#[test]
fn a_whole_column_and_the_sheet_edge_stop_the_range() {
    assert_results(&[
        ("=SUMIF(A:A,\"\",C1)", "111"),
        ("=SUMIF(A:A,\"\",C2)", "110"),
        ("=AVERAGEIF(A:A,\"\",C2)", "55"),
        ("=SUMIF(A:A,\"\",C:C)", "111"),
        ("=SUMIF(B1:B2,5,C1048576)", "0"),
        ("=SUMIF(B1:B2,5,C1048575)", "0"),
        ("=SUMIF(B1:C1,\"text\",XFD1)", "0"),
        // A whole row: the range stops at the last column, then at the used range.
        ("=SUMIF(A3:XFD3,100,B1)", "1000"),
        ("=SUMIF(B2:XFD2,5,C2)", "10"),
    ]);
}

#[test]
fn other_criteria_results_stay() {
    assert_results(&[
        ("=SUMIFS(C1,B1:B2,5)", "#VALUE!"),
        ("=AVERAGEIFS(C1,B1:B2,5)", "#VALUE!"),
        ("=SUMIF(B1:B2,5,C1:C2)", "10"),
        ("=SUMIF(B1:B2,5)", "5"),
        ("=AVERAGEIF(B1:B2,5,C1:C2)", "10"),
        ("=SUMIF(B1:B2,5,OFFSET(A1,-1,0))", "#REF!"),
        ("=SUMIF(B1:B2,{5},OFFSET(A1,-1,0))", "#REF!"),
    ]);
}
