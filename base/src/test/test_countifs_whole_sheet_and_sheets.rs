#![allow(clippy::unwrap_used)]

//! IronXL: COUNTIF and COUNTIFS count each empty cell of a whole-sheet range without a wrap, and
//! read the used range of each criteria range on its own sheet, as in Excel.

use crate::cell::CellValue;
use crate::test::util::new_empty_model;

/// The value of `formula` in Sheet3!A1, outside each counted range, over the `values` of Sheet1 and
/// Sheet2.
fn value(values: &[(&str, &str)], formula: &str) -> CellValue {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_sheet();
    for (reference, input) in values {
        model._set(reference, input);
    }
    model._set("Sheet3!A1", formula);
    model.evaluate();
    model.get_cell_value_by_ref("Sheet3!A1").unwrap()
}

/// The value of `formula` over Sheet1!A1 = 1, Sheet2!A10 = "x", and Sheet2!AC1 = "y".
fn two_sheet_value(formula: &str) -> CellValue {
    value(
        &[("Sheet1!A1", "1"), ("Sheet2!A10", "x"), ("Sheet2!AC1", "y")],
        formula,
    )
}

/// The value of `formula` over Sheet2!A10 = "x" and no other value.
fn second_sheet_value(formula: &str) -> CellValue {
    value(&[("Sheet2!A10", "x")], formula)
}

/// The value of `formula` over Sheet1!A10 = "x" and no other value.
fn first_sheet_value(formula: &str) -> CellValue {
    value(&[("Sheet1!A10", "x")], formula)
}

fn number(value: f64) -> CellValue {
    CellValue::Number(value)
}

#[test]
fn countif_counts_each_empty_cell_of_a_whole_sheet_on_another_sheet() {
    assert_eq!(
        second_sheet_value("=COUNTIF(Sheet2!A:XFD,\"\")"),
        number(17179869183.0)
    );
    assert_eq!(
        second_sheet_value("=COUNTIF(Sheet2!A:XFD,\"<>5\")"),
        number(17179869184.0)
    );
    assert_eq!(
        second_sheet_value("=COUNTIF(Sheet2!A:XFD,\"<>x\")"),
        number(17179869183.0)
    );
    assert_eq!(
        second_sheet_value("=COUNTIFS(Sheet2!A:XFD,\"\")"),
        number(17179869183.0)
    );
    assert_eq!(
        second_sheet_value("=COUNTIF(Sheet2!A:XFD,\"x\")"),
        number(1.0)
    );
}

#[test]
fn countif_counts_each_empty_cell_of_a_whole_sheet_on_the_first_sheet() {
    assert_eq!(
        first_sheet_value("=COUNTIF(Sheet1!A:XFD,\"\")"),
        number(17179869183.0)
    );
    assert_eq!(
        first_sheet_value("=COUNTIF(Sheet1!A:XFD,\"<>5\")"),
        number(17179869184.0)
    );
    assert_eq!(
        first_sheet_value("=COUNTIF(Sheet1!A:XFD,\"<>x\")"),
        number(17179869183.0)
    );
    assert_eq!(
        first_sheet_value("=COUNTIF(Sheet1!A:XFD,\"<>\")"),
        number(1.0)
    );
}

#[test]
fn countif_counts_the_empty_cells_of_whole_rows() {
    assert_eq!(
        first_sheet_value("=COUNTIF(Sheet1!1:1,\"\")"),
        number(16384.0)
    );
    assert_eq!(
        two_sheet_value("=COUNTIF(Sheet2!1:1,\"\")"),
        number(16383.0)
    );
}

#[test]
fn countifs_reads_each_whole_column_on_its_own_sheet() {
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!A:A,\"\",Sheet2!A:A,\"<>\")"),
        number(1.0)
    );
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!A:A,\"<>x\",Sheet2!A:A,\"<>x\")"),
        number(1048575.0)
    );
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet2!A:A,\"<>\",Sheet1!A:A,\"\")"),
        number(1.0)
    );
}

#[test]
fn countifs_reads_each_whole_row_on_its_own_sheet() {
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!1:1,\"\",Sheet2!1:1,\"<>\")"),
        number(1.0)
    );
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!1:1,\"<>y\",Sheet2!1:1,\"<>y\")"),
        number(16383.0)
    );
}

#[test]
fn countifs_on_one_sheet_keeps_its_results() {
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!A:A,\"\",Sheet1!A:A,\"<>1\")"),
        number(1048575.0)
    );
    assert_eq!(
        two_sheet_value("=COUNTIFS(Sheet1!A:A,1,Sheet1!A:A,\"<>x\")"),
        number(1.0)
    );
    assert_eq!(two_sheet_value("=COUNTIFS(Sheet1!A1:A3,\"\")"), number(2.0));
}

#[test]
fn sumifs_across_sheets_keeps_its_result() {
    assert_eq!(
        two_sheet_value("=SUMIFS(Sheet1!A:A,Sheet2!A:A,\"<>\")"),
        number(0.0)
    );
    assert_eq!(
        two_sheet_value("=SUMIFS(Sheet1!A:A,Sheet2!A:A,\"\")"),
        number(1.0)
    );
}
