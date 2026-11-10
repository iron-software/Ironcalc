#![allow(clippy::unwrap_used)]

//! IronXL: a defined name before `:` and a sheet-qualified reference after it, such as
//! `Remote:Sheet2!A2`, is the range operator between the name and the reference, as in Excel.
//! The text is a 3-D reference only when its first end is a sheet, or when no defined name in
//! scope has that text. The other rules of the range operator apply to the two sides.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1`, `Sheet2`, `Data1`, and `Data3`. `Sheet1!A1:A2` is 1, 2;
/// `Sheet2!A1:A3` is 100, 200, 300; `Data1!A2` is 5 and `Data3!A2` is 7. The workbook names are
/// `Remote` (`Sheet2!$A$1`), `RemoteBlock` (`Sheet2!$A$1:$A$2`), `Local` (`Sheet1!$A$1`), `Rate`
/// (0.2), and `Data1` (`Sheet2!$A$3`). The name `Here` of `Sheet2` is `Sheet2!$A$1`.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    for sheet in ["Sheet2", "Data1", "Data3"] {
        model.add_sheet(sheet).unwrap();
    }
    for (reference, input) in [
        ("Sheet1!A1", "1"),
        ("Sheet1!A2", "2"),
        ("Sheet2!A1", "100"),
        ("Sheet2!A2", "200"),
        ("Sheet2!A3", "300"),
        ("Data1!A2", "5"),
        ("Data3!A2", "7"),
    ] {
        model._set(reference, input);
    }
    for result in model.add_defined_names(&[
        ("Remote", None, "Sheet2!$A$1"),
        ("RemoteBlock", None, "Sheet2!$A$1:$A$2"),
        ("Local", None, "Sheet1!$A$1"),
        ("Rate", None, "0.2"),
        ("Data1", None, "Sheet2!$A$3"),
        ("Here", Some(1), "Sheet2!$A$1"),
    ]) {
        result.unwrap();
    }
    model
}

/// Checks that each formula, in column `P` of `sheet`, gives its expected text.
fn assert_results(sheet: &str, cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("{sheet}!P{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("{sheet}!P{}", index + 1)),
            *expected,
            "{sheet}: {formula}"
        );
    }
}

#[test]
fn a_name_before_a_reference_on_its_sheet_gives_the_range_on_that_sheet() {
    assert_results(
        "Sheet1",
        &[
            ("=SUM(Remote:Sheet2!A2)", "300"),
            ("=ROWS(Remote:Sheet2!A2)", "2"),
            ("=SUM(RemoteBlock:Sheet2!A3)", "600"),
            ("=SUM(remote:Sheet2!A2)", "300"),
            ("=SUM(REMOTE:Sheet2!A2)", "300"),
            ("=ISREF(Remote:Sheet2!A2)", "TRUE"),
            ("=SUM(Remote:Sheet2!$A$3)", "600"),
            ("=SUM(Remote:Sheet2!B3)", "600"),
            ("=COLUMNS(Remote:Sheet2!B3)", "2"),
            ("=SUM( Remote:Sheet2!A2 , 1)", "301"),
        ],
    );
}

#[test]
fn a_name_before_a_reference_on_the_sheet_of_the_formula_gives_the_range() {
    assert_results(
        "Sheet2",
        &[
            ("=SUM(Remote:Sheet2!A2)", "300"),
            ("=SUM(Remote:A2)", "300"),
            ("=SUM(Here:Sheet2!A2)", "300"),
            ("=SUM(here:Sheet2!A3)", "600"),
        ],
    );
}

#[test]
fn the_rules_of_the_range_operator_apply_to_a_name_side() {
    assert_results(
        "Sheet1",
        &[
            ("=SUM(Local:Sheet2!A2)", "#VALUE!"),
            ("=SUM(Remote:Sheet1!A2)", "#VALUE!"),
            ("=SUM(Rate:Sheet2!A2)", "#VALUE!"),
        ],
    );
    assert_results("Sheet2", &[("=SUM(Remote:Sheet1!A2)", "#VALUE!")]);
}

#[test]
fn a_name_of_another_sheet_scope_does_not_hide_a_3d_reference() {
    // `Here` is a name of `Sheet2`, so on `Sheet1` the text is a 3-D reference with an end that
    // is not a sheet.
    assert_results("Sheet1", &[("=SUM(Here:Sheet2!A2)", "#REF!")]);
}

#[test]
fn a_3d_reference_keeps_its_result_when_a_name_has_the_text_of_an_end() {
    assert_results(
        "Sheet1",
        &[
            ("=SUM(Data1:Data3!A2)", "12"),
            ("=SUM(data1:Data3!A2)", "12"),
            ("=SUM(Nope:Sheet2!A2)", "#REF!"),
            ("=SUM('Remote:Sheet2'!A2)", "#REF!"),
        ],
    );
}

#[test]
fn the_text_of_a_name_range_parses_again_with_the_same_result() {
    let mut model = model();
    model._set("P1", "=SUM(Remote:Sheet2!A2)");
    model.evaluate();
    let text = model._get_formula("P1");
    assert_eq!(text, "=SUM(Remote:Sheet2!A2)");
    model._set("P2", &text);
    model.evaluate();
    assert_eq!(model._get_text("P2"), "300");
}

#[test]
fn a_name_range_follows_a_change_of_a_cell_in_the_range() {
    let mut model = model();
    model._set("P1", "=SUM(Remote:Sheet2!A2)");
    model.evaluate();
    assert_eq!(model._get_text("P1"), "300");
    model._set("Sheet2!A2", "250");
    model.evaluate();
    assert_eq!(model._get_text("P1"), "350");
}
