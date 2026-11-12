#![allow(clippy::unwrap_used)]

//! IronXL: a sheet prefix finds its sheet without regard to letter case, as in Excel. So
//! `=SUM(sheet2!A1:A2)` reads the sheet `Sheet2`. The formula text keeps the prefix as written.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with Sheet1 E1=1, F1=2, E2=3, F2=4, Sheet2 A1=100, A2=200, a sheet `Äpfel` with A1=5,
/// and the workbook name `Remote` for `Sheet2!$A$1`.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.new_sheet();
    model.add_sheet("Äpfel").unwrap();
    for result in model.add_defined_names(&[("Remote", None, "Sheet2!$A$1")]) {
        result.unwrap();
    }
    for (reference, input) in [
        ("Sheet1!E1", "1"),
        ("Sheet1!F1", "2"),
        ("Sheet1!E2", "3"),
        ("Sheet1!F2", "4"),
        ("Sheet2!A1", "100"),
        ("Sheet2!A2", "200"),
    ] {
        model._set(reference, input);
    }
    model.set_user_input(2, 1, 1, "5".to_string()).unwrap();
    model
}

/// The value of `formula` in `cell` of the model above.
fn value_at(cell: &str, formula: &str) -> CellValue {
    let mut model = model();
    model._set(cell, formula);
    model.evaluate();
    model.get_cell_value_by_ref(cell).unwrap()
}

/// The value of `formula` in Sheet1!Z1 of the model above.
fn value(formula: &str) -> CellValue {
    value_at("Sheet1!Z1", formula)
}

fn error(text: &str) -> CellValue {
    CellValue::String(text.to_string())
}

#[test]
fn a_range_with_a_prefix_in_another_case_reads_the_sheet() {
    assert_eq!(value("=SUM(sheet2!A1:A2)"), CellValue::Number(300.0));
    assert_eq!(value("=SUM(SHEET2!A1:A2)"), CellValue::Number(300.0));
}

#[test]
fn the_range_operator_with_prefixes_in_another_case_reads_the_sheet() {
    assert_eq!(value("=SUM(Sheet2!A1:sheet2!A2)"), CellValue::Number(300.0));
    assert_eq!(value("=SUM(sheet2!A1:sheet2!A2)"), CellValue::Number(300.0));
    assert_eq!(value("=SUM(sheet1!E1:sheet1!F2)"), CellValue::Number(10.0));
}

#[test]
fn a_name_and_a_prefix_in_another_case_make_a_range() {
    assert_eq!(value("=SUM(Remote:sheet2!A2)"), CellValue::Number(300.0));
    assert_eq!(value("=SUM(Remote:SHEET2!A2)"), CellValue::Number(300.0));
    assert_eq!(value("=SUM(sheet2!A2:Remote)"), CellValue::Number(300.0));
    // `Remote` is Sheet2!$A$1, so the range is A1:A1, as in Excel.
    assert_eq!(value("=SUM(sheet2!A1:Remote)"), CellValue::Number(100.0));
}

#[test]
fn a_reference_with_a_prefix_in_another_case_reads_the_sheet() {
    assert_eq!(value("=sheet1!E1"), CellValue::Number(1.0));
    assert_eq!(value("=sheet2!$A$2"), CellValue::Number(200.0));
    assert_eq!(value("='sheet2'!A1"), CellValue::Number(100.0));
    assert_eq!(
        value("=SUM(sheet1!E1:OFFSET(F2,0,0))"),
        CellValue::Number(10.0)
    );
}

#[test]
fn a_whole_column_with_a_prefix_in_another_case_reads_the_sheet() {
    assert_eq!(value("=SUM(SHEET2!A:A)"), CellValue::Number(300.0));
}

#[test]
fn a_prefix_in_another_case_of_the_own_sheet_reads_the_sheet() {
    assert_eq!(
        value_at("Sheet2!Z1", "=SUM(sheet2!A1:A2)"),
        CellValue::Number(300.0)
    );
}

#[test]
fn a_non_ascii_prefix_in_another_case_reads_the_sheet() {
    assert_eq!(value("=äPFEL!A1"), CellValue::Number(5.0));
    assert_eq!(value("='äpfel'!A1"), CellValue::Number(5.0));
}

#[test]
fn an_unknown_sheet_and_two_sheets_keep_their_errors() {
    assert_eq!(value("=nosheet!A1"), error("#REF!"));
    assert_eq!(value("=SUM(nosheet!A1:A2)"), error("#REF!"));
    assert_eq!(value("=SUM(sheet1!E1:sheet2!A2)"), error("#VALUE!"));
}

#[test]
fn the_formula_keeps_the_prefix_as_written() {
    let mut model = model();
    model._set("Sheet1!Z1", "=SUM(sheet2!A1:A2)");
    model.evaluate();
    assert_eq!(
        model.get_cell_formula(0, 1, 26).unwrap(),
        Some("=SUM(sheet2!A1:A2)".to_string())
    );
}

#[test]
fn a_rename_of_the_sheet_changes_a_prefix_in_another_case() {
    let mut model = model();
    model._set("Sheet1!Z1", "=SUM(sheet2!A1:A2)");
    model.evaluate();
    model.rename_sheet("Sheet2", "Data").unwrap();
    model.evaluate();
    assert_eq!(
        model.get_cell_formula(0, 1, 26).unwrap(),
        Some("=SUM(Data!A1:A2)".to_string())
    );
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!Z1").unwrap(),
        CellValue::Number(300.0)
    );
}

#[test]
fn a_three_d_reference_keeps_its_result() {
    let mut model = model();
    model._set("Sheet1!Z1", "=SUM(SHEET1:sheet2!A2)");
    model.evaluate();
    assert_eq!(
        model.get_cell_value_by_ref("Sheet1!Z1").unwrap(),
        CellValue::Number(200.0)
    );
}
