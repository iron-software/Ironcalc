#![allow(clippy::unwrap_used)]

//! IronXL: a sheet-qualified reference or a reference with `$` is the left side of the range
//! operator when the text after the `:` is not a cell reference: a function call, a name, or
//! another sheet-qualified reference. So `Sheet1!E1:OFFSET(F2,0,0)` and `$E$1:OFFSET(F2,0,0)`
//! give the results of `E1:OFFSET(F2,0,0)`. The A1 and the R1C1 lexer give the same tokens, so
//! a formula keeps its result after the engine parses its R1C1 text again.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1`, `Sheet2`, and `Other`. `Sheet1!E1` is 1, `F1` is 2, `E2` is 3, `F2` is
/// 4, `C1` is 7, and `C2` is 8. `Sheet2!A1:A3` is 100, 200, and 300, and `Sheet2!E1` is 100.
/// The workbook name `Remote` is `Sheet2!$A$1`.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model.add_sheet("Other").unwrap();
    for (reference, value) in [
        ("E1", "1"),
        ("F1", "2"),
        ("E2", "3"),
        ("F2", "4"),
        ("C1", "7"),
        ("C2", "8"),
        ("Sheet2!A1", "100"),
        ("Sheet2!A2", "200"),
        ("Sheet2!A3", "300"),
        ("Sheet2!E1", "100"),
    ] {
        model._set(reference, value);
    }
    for result in model.add_defined_names(&[("Remote", None, "Sheet2!$A$1")]) {
        result.unwrap();
    }
    model
}

/// Sets each formula in column `H` of `sheet`, from row 1, and evaluates the model.
fn set_formulas(model: &mut Model, sheet: &str, cases: &[(&str, &str)]) {
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("{sheet}!H{}", index + 1), formula);
    }
    model.evaluate();
}

/// Checks that each formula in column `H` of `sheet` gives its expected text.
fn assert_texts(model: &Model, sheet: &str, cases: &[(&str, &str)]) {
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("{sheet}!H{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

/// Checks that each formula, in column `H` of `Sheet1`, gives its expected text.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    set_formulas(&mut model, "Sheet1", cases);
    assert_texts(&model, "Sheet1", cases);
}

#[test]
fn a_sheet_qualified_reference_before_a_function_call_is_the_range_operator() {
    // Fails before the lexer change: each formula gave #ERROR!.
    assert_results(&[
        ("=SUM(Sheet1!E1:OFFSET(F2,0,0))", "10"),
        ("=SUM(Sheet1!$E$1:INDEX(Sheet1!$E:$E,2))", "4"),
        ("=SUM(Sheet2!E1:OFFSET(A1,0,0))", "#VALUE!"),
        ("=SUM(Sheet2!A2:INDEX(Sheet2!A1:A3,3))", "500"),
        ("=SUM('Sheet2'!A2:INDEX(Sheet2!A1:A3,3))", "500"),
    ]);
}

#[test]
fn a_reference_with_dollar_before_a_function_call_is_the_range_operator() {
    // Fails before the lexer change: each formula gave #ERROR!.
    assert_results(&[
        ("=SUM($E$1:OFFSET(F2,0,0))", "10"),
        ("=SUM($E1:OFFSET(F2,0,0))", "10"),
        ("=SUM(E$1:OFFSET(F2,0,0))", "10"),
        ("=SUM($E$1:OFFSET(F2,0,0):C1)", "25"),
    ]);
}

#[test]
fn a_sheet_qualified_reference_before_a_name_or_a_sheet_reference_is_the_range_operator() {
    // Fails before the lexer change: each formula gave #ERROR!.
    let cases = [
        ("=SUM(Sheet2!A2:Remote)", "300"),
        ("=ROWS(Sheet2!A3:Remote)", "3"),
        ("=SUM(Sheet2!A1:Sheet2!A2)", "300"),
        ("=SUM(Sheet1!E1:Sheet2!A2)", "#VALUE!"),
    ];
    assert_results(&cases);
    let mut model = model();
    let on_sheet2 = [("=SUM(Sheet2!A2:Remote)", "300")];
    set_formulas(&mut model, "Sheet2", &on_sheet2);
    assert_texts(&model, "Sheet2", &on_sheet2);
}

#[test]
fn the_forms_that_parsed_before_keep_their_results() {
    assert_results(&[
        ("=SUM(Sheet1!E1:F2)", "10"),
        ("=SUM(Sheet1!E:F)", "10"),
        ("=SUM($E$1:F2)", "10"),
        ("=SUM(E1:OFFSET(F2,0,0))", "10"),
        ("=SUM(E1:OFFSET(F2,0,0):$C$1)", "25"),
        ("=SUM(Sheet2!A1:A3)", "600"),
    ]);
}

#[test]
fn a_reference_that_ends_in_a_range_operator_does_not_parse() {
    assert_results(&[
        ("=SUM(Sheet1!E1:)", "#ERROR!"),
        ("=SUM($E$1:)", "#ERROR!"),
        ("=SUM('Sheet2'!A1:)", "#ERROR!"),
    ]);
}

#[test]
fn a_sheet_reference_range_keeps_its_result_and_its_text_after_a_rename() {
    // Each rename parses the R1C1 text of every formula again. Fails with a change of the A1
    // lexer path only: the formulas gave #ERROR! and their R1C1 text after the rename.
    let mut model = model();
    let cases = [
        ("=SUM(Sheet1!E1:OFFSET(F2,0,0))", "10"),
        ("=SUM($E$1:OFFSET(F2,0,0))", "10"),
        ("=SUM(Sheet2!A2:Remote)", "300"),
        ("=SUM(Sheet2!A1:Sheet2!A2)", "300"),
        ("=SUM(Sheet2!A2:INDEX(Sheet2!A1:A3,3))", "500"),
    ];
    set_formulas(&mut model, "Sheet1", &cases);
    model.rename_sheet("Other", "Renamed").unwrap();
    model.evaluate();
    assert_texts(&model, "Sheet1", &cases);
    for (row, (formula, _)) in cases.iter().enumerate() {
        assert_eq!(
            model
                .get_cell_formula(0, row as i32 + 1, 8)
                .unwrap()
                .as_deref(),
            Some(*formula)
        );
    }
    model.rename_sheet("Sheet2", "Data").unwrap();
    model.evaluate();
    assert_texts(&model, "Sheet1", &cases);
    let renamed = [
        "=SUM(Sheet1!E1:OFFSET(F2,0,0))",
        "=SUM($E$1:OFFSET(F2,0,0))",
        "=SUM(Data!A2:Remote)",
        "=SUM(Data!A1:Data!A2)",
        "=SUM(Data!A2:INDEX(Data!A1:A3,3))",
    ];
    for (row, formula) in renamed.iter().enumerate() {
        assert_eq!(
            model
                .get_cell_formula(0, row as i32 + 1, 8)
                .unwrap()
                .as_deref(),
            Some(*formula)
        );
    }
}

#[test]
fn a_sheet_reference_range_follows_a_change_of_its_inputs() {
    let mut model = model();
    let cases = [("=SUM(Sheet1!E1:OFFSET(F2,0,0))", "10")];
    set_formulas(&mut model, "Sheet1", &cases);
    model._set("F2", "40");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "46");
}
