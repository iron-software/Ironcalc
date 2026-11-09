#![allow(clippy::unwrap_used)]

//! IronXL: each criteria range of COUNTIFS, SUMIFS, AVERAGEIFS, MINIFS, and MAXIFS must have the
//! rows and columns of the first range, else the function gives `#VALUE!`, as in Excel. COUNTIF,
//! SUMIF, and AVERAGEIF give no size error.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!B1` text and `B2` 5. The other cells are empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model.
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
fn countifs_and_sumifs_give_value_for_ranges_of_different_sizes() {
    assert_results(&[
        ("=COUNTIFS(B2:B2,5,B1:B2,\"text\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,5,B1:B3,\"text\")", "#VALUE!"),
        ("=SUMIFS(B1:B2,B1:B3,\"text\")", "#VALUE!"),
        ("=COUNTIFS(INDEX(B1:B2,2),5,B1:B2,\"text\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,\"*\",C1:C3,\"\")", "#VALUE!"),
        ("=COUNTIFS(B2:C2,5,B1:B2,\"text\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,5,X1,\"x\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,5,Sheet1!B1:B3,\"x\")", "#VALUE!"),
    ]);
}

#[test]
fn the_other_ifs_functions_compare_rows_and_columns() {
    assert_results(&[
        ("=AVERAGEIFS(B1:B2,B1:B3,\"text\")", "#VALUE!"),
        ("=MINIFS(B1:B2,B1:B3,\"text\")", "#VALUE!"),
        ("=MAXIFS(B1:B2,B1:B3,\"text\")", "#VALUE!"),
        // Two cells in each range, but 2 rows and 1 column against 1 row and 2 columns.
        ("=SUMIFS(B1:B2,B1:C1,\"text\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,\"text\",B1:C1,\"text\")", "#VALUE!"),
        // Four cells in each range, but 2 by 2 against 4 by 1.
        ("=SUMIFS(B1:C2,B1:B4,\"text\")", "#VALUE!"),
        ("=SUMIFS(B1:B2,B1:C2,\"text\")", "#VALUE!"),
        // A whole column compares its declared rows, not the used range.
        ("=SUMIFS(B:B,A1:A5,\"\")", "#VALUE!"),
        ("=COUNTIFS(A:A,\"\",B1:B10,\"\")", "#VALUE!"),
        ("=COUNTIFS(A1:A2,\"\",B:B,\"\")", "#VALUE!"),
        // A later criteria pair.
        ("=SUMIFS(B1:B2,B1:B2,\"text\",C1:C3,\"\")", "#VALUE!"),
        ("=COUNTIFS(B1:B2,\"text\",C1:C2,\"\",D1:D3,\"\")", "#VALUE!"),
    ]);
}

#[test]
fn an_error_of_a_range_argument_comes_before_the_size_error() {
    assert_results(&[
        ("=COUNTIFS(B1:B2,5,OFFSET(A1,-1,0),\"x\")", "#REF!"),
        (
            "=SUMIFS(B1:B2,B1:B3,\"text\",OFFSET(A1,-1,0),\"x\")",
            "#REF!",
        ),
        ("=SUMIFS(OFFSET(A1,-1,0),B1:B3,\"text\")", "#REF!"),
        // A criterion error is not a range error, so the size error stays.
        ("=COUNTIFS(B1:B2,1/0,B1:B3,\"x\")", "#VALUE!"),
    ]);
}

#[test]
fn ranges_of_the_same_size_keep_their_results() {
    assert_results(&[
        ("=COUNTIFS(B1:B2,\"text\",C1:C2,\"\")", "1"),
        ("=COUNTIFS(B1:B2,5,B1:B2,\"<>x\")", "1"),
        ("=COUNTIFS(A:A,\"\",Sheet1!B:B,\"\")", "1048574"),
        ("=COUNTIFS(B2,5)", "1"),
        ("=SUMIFS(B:B,A:A,\"\")", "5"),
        ("=SUMIFS(B1:B2,C1:C2,\"\")", "5"),
        ("=SUMIFS(B1:C2,B1:C2,5)", "5"),
        ("=AVERAGEIFS(B1:B2,C1:C2,\"\")", "5"),
        ("=MINIFS(B1:B2,C3:C4,\"\")", "5"),
        ("=MAXIFS(B1:B2,B1:B2,\"<>text\",C1:C2,\"\")", "5"),
    ]);
}

#[test]
fn countif_sumif_and_averageif_give_no_size_error() {
    assert_results(&[
        ("=SUMIF(B1:B2,5,B1:B5)", "5"),
        ("=AVERAGEIF(B1:B2,5,B1:B5)", "5"),
        ("=SUMIF(B1:B3,\"<>\",C1:C2)", "0"),
        ("=COUNTIF(B1:B3,\"\")", "1"),
        ("=SUMIF(B1:B2,5)", "5"),
        ("=AVERAGEIF(B1:B2,5)", "5"),
        ("=SUMIF(B1:B2,{5,\"text\"},B1:B5)", "5"),
    ]);
}
