#![allow(clippy::unwrap_used)]

//! IronXL: a defined name can hold a constant or a formula, such as `OFFSET(...)`. The engine
//! evaluates it in the context of the calling cell, and a name of the sheet of the calling cell
//! shadows a workbook name, as in Excel.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with the `names` added, then the `inputs` set, then evaluated.
fn model_with<'a>(
    sheets: usize,
    names: &[(&str, Option<u32>, &str)],
    inputs: &[(&str, &str)],
) -> Model<'a> {
    let mut model = new_empty_model();
    for _ in 1..sheets {
        model.new_sheet();
    }
    for result in model.add_defined_names(names) {
        result.unwrap();
    }
    for (reference, input) in inputs {
        model._set(reference, input);
    }
    model.evaluate();
    model
}

/// The number in a cell, or NaN for any other value, so a comparison with a number fails.
fn number(model: &Model, reference: &str) -> f64 {
    match model.get_cell_value_by_ref(reference).unwrap() {
        CellValue::Number(number) => number,
        _ => f64::NAN,
    }
}

#[test]
fn a_constant_name_gives_its_value() {
    let model = model_with(
        1,
        &[
            ("TaxRate", None, "0.2"),
            ("Greeting", None, "=\"Hello\""),
            ("Flag", None, "TRUE"),
        ],
        &[
            ("A1", "100"),
            ("B1", "=A1*TaxRate"),
            ("B2", "=Greeting&\" world\""),
            ("B3", "=IF(Flag,1,2)"),
            ("B4", "=taxrate"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!B1"), 20.0);
    assert_eq!(model._get_text("B2"), "Hello world");
    assert_eq!(number(&model, "Sheet1!B3"), 1.0);
    assert_eq!(number(&model, "Sheet1!B4"), 0.2);
    assert_eq!(model._get_formula("B1"), "=A1*TaxRate");
}

#[test]
fn a_formula_name_follows_its_inputs() {
    let mut model = model_with(
        1,
        &[("Doubled", None, "Sheet1!$A$1*2")],
        &[("A1", "5"), ("B1", "=Doubled+1")],
    );
    assert_eq!(number(&model, "Sheet1!B1"), 11.0);
    model._set("A1", "7");
    model.evaluate();
    assert_eq!(number(&model, "Sheet1!B1"), 15.0);
}

#[test]
fn an_offset_name_is_a_dynamic_range_that_follows_its_inputs() {
    let mut model = model_with(
        1,
        &[("Window", None, "OFFSET(Sheet1!$A$1,0,0,Sheet1!$B$1,1)")],
        &[
            ("A1", "1"),
            ("A2", "2"),
            ("A3", "3"),
            ("A4", "4"),
            ("A5", "5"),
            ("B1", "3"),
            ("C1", "=SUM(Window)"),
            ("C2", "=ROWS(Window)"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!C1"), 6.0);
    assert_eq!(number(&model, "Sheet1!C2"), 3.0);
    model._set("B1", "5");
    // A partial evaluation reads the inputs of the name, because its references are dynamic.
    model.evaluate_cells(&[(0, 1, 3), (0, 2, 3)]).unwrap();
    assert_eq!(number(&model, "Sheet1!C1"), 15.0);
    assert_eq!(number(&model, "Sheet1!C2"), 5.0);
    assert_eq!(model._get_formula("C1"), "=SUM(Window)");
}

#[test]
fn an_offset_name_is_evaluated_at_the_calling_cell() {
    // The name is read at A1, so `A1` in its text is the calling cell, and `A2` the cell below.
    let model = model_with(
        1,
        &[
            ("Above", None, "OFFSET(Sheet1!A1,-1,0)"),
            ("Below", None, "Sheet1!A2*1"),
        ],
        &[
            ("C4", "10"),
            ("C5", "=Above+1"),
            ("E9", "=Above+2"),
            ("E8", "20"),
            ("G1", "=Below"),
            ("G2", "=Below"),
            ("G3", "30"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!C5"), 11.0);
    assert_eq!(number(&model, "Sheet1!E9"), 22.0);
    // G1 reads G2, which reads G3 through the same name.
    assert_eq!(number(&model, "Sheet1!G1"), 30.0);
    assert_eq!(number(&model, "Sheet1!G2"), 30.0);
}

#[test]
fn a_relative_reference_name_counts_from_the_calling_cell() {
    // As in Excel, a relative reference in the text of a name counts from A1, and the engine
    // evaluates it at the calling cell. An absolute reference stays where it is.
    let model = model_with(
        1,
        &[
            ("Next", None, "Sheet1!B2"),
            ("Column", None, "Sheet1!B:B"),
            ("Fixed", None, "Sheet1!$B$2"),
            ("Fixed_column", None, "Sheet1!$A:$A"),
        ],
        &[
            ("B2", "5"),
            ("D4", "7"),
            ("H1", "1"),
            ("H2", "2"),
            ("A1", "10"),
            ("C3", "=Next"),
            ("G10", "=SUM(Column)"),
            ("E1", "=Fixed"),
            ("E2", "=SUM(Fixed_column)"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!C3"), 7.0);
    // `B:B` at G10 is the column to the right of the calling cell.
    assert_eq!(number(&model, "Sheet1!G10"), 3.0);
    assert_eq!(number(&model, "Sheet1!E1"), 5.0);
    assert_eq!(number(&model, "Sheet1!E2"), 10.0);
}

#[test]
fn a_sheet_name_shadows_the_workbook_name_on_its_sheet() {
    let model = model_with(
        3,
        &[
            ("Rate", None, "0.1"),
            ("Rate", Some(1), "0.5"),
            ("Twice", None, "Rate*2"),
            ("Twice", Some(2), "Rate*3"),
        ],
        &[
            ("Sheet1!A1", "=Rate"),
            ("Sheet2!A1", "=Rate"),
            ("Sheet3!A1", "=Rate"),
            ("Sheet1!A2", "=Twice"),
            ("Sheet2!A2", "=Twice"),
            ("Sheet3!A2", "=Twice"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!A1"), 0.1);
    assert_eq!(number(&model, "Sheet2!A1"), 0.5);
    assert_eq!(number(&model, "Sheet3!A1"), 0.1);
    // The text of a workbook name resolves workbook names only, also on a sheet with its own `Rate`.
    assert_eq!(number(&model, "Sheet1!A2"), 0.2);
    assert_eq!(number(&model, "Sheet2!A2"), 0.2);
    assert_eq!(number(&model, "Sheet3!A2"), 0.30000000000000004);
}

#[test]
fn the_text_of_a_sheet_name_resolves_on_its_sheet_first() {
    let model = model_with(
        2,
        &[
            ("Rate", None, "0.1"),
            ("Rate", Some(1), "0.5"),
            ("Twice", Some(1), "Rate*2"),
        ],
        &[("Sheet2!A1", "=Twice")],
    );
    assert_eq!(number(&model, "Sheet2!A1"), 1.0);
}

#[test]
fn a_name_that_refers_to_itself_is_a_circular_reference() {
    let model = model_with(
        1,
        &[
            ("Loop", None, "Loop+1"),
            ("Ping", None, "Pong"),
            ("Pong", None, "Ping*1"),
        ],
        &[("A1", "=Loop"), ("A2", "=Ping")],
    );
    assert_eq!(model._get_text("A1"), "#CIRC!");
    assert_eq!(model._get_text("A2"), "#CIRC!");
}

#[test]
fn a_formula_after_a_removed_name_gives_name_error() {
    let mut model = model_with(
        1,
        &[("Rate", None, "0.2")],
        &[("A1", "=Rate"), ("A2", "=Missing")],
    );
    assert_eq!(number(&model, "Sheet1!A1"), 0.2);
    assert_eq!(model._get_text("A2"), "#NAME?");
    model.delete_defined_name("Rate", None).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#NAME?");
    assert_eq!(model._get_formula("A1"), "=Rate");
}

#[test]
fn sheet_of_a_formula_name() {
    let model = model_with(
        2,
        &[
            ("Window", None, "OFFSET(Sheet2!$A$1,0,0,2,1)"),
            ("Rate", None, "0.2"),
        ],
        &[("A1", "=SHEET(Window)"), ("A2", "=SHEET(Rate)")],
    );
    assert_eq!(number(&model, "Sheet1!A1"), 2.0);
    assert_eq!(model._get_text("A2"), "#ERROR!");
}

#[test]
fn add_defined_names_refuses_a_bad_name_and_adds_the_others() {
    let mut model = new_empty_model();
    model.new_defined_name("Rate", None, "Sheet1!$A$1").unwrap();
    let results = model.add_defined_names(&[
        ("rate", None, "0.5"),
        ("Other", Some(3), "1"),
        ("Broken", None, "SUM(1"),
        ("Good", None, "=2"),
        ("good", None, "3"),
        ("Back\\slash", None, "4"),
        ("Rate", Some(0), "5"),
    ]);
    assert_eq!(
        results,
        vec![
            Err("Name: Defined name already exists".to_string()),
            Err("Scope: Invalid sheet index".to_string()),
            Err("Formula: Invalid defined name formula".to_string()),
            Ok(()),
            Err("Name: Defined name already exists".to_string()),
            Ok(()),
            Ok(()),
        ]
    );
    assert_eq!(
        model.get_defined_name_list(),
        vec![
            ("Rate".to_string(), None, "Sheet1!$A$1".to_string()),
            ("Good".to_string(), None, "=2".to_string()),
            ("Back\\slash".to_string(), None, "4".to_string()),
            ("Rate".to_string(), Some(0), "5".to_string()),
        ]
    );
    model._set("A2", "=Good+Rate");
    model.evaluate();
    assert_eq!(number(&model, "Sheet1!A2"), 7.0);
}

#[test]
fn add_defined_name_adds_one_name() {
    let mut model = new_empty_model();
    model._set("A1", "=Rate*10");
    model.add_defined_name("Rate", None, "0.25").unwrap();
    assert_eq!(number(&model, "Sheet1!A1"), 2.5);
    assert_eq!(
        model.add_defined_name("RATE", None, "1"),
        Err("Name: Defined name already exists".to_string())
    );
    // A refused name changes nothing.
    assert_eq!(model.add_defined_names(&[("X", None, "(")]).len(), 1);
    assert_eq!(model.get_defined_name_list().len(), 1);
}

#[test]
fn a_lambda_and_a_reference_added_by_add_defined_name_work_as_before() {
    let model = model_with(
        1,
        &[
            ("Square", None, "LAMBDA(x,x*x)"),
            ("Cell", None, "Sheet1!$A$1"),
            ("Cells", None, "Sheet1!$A$1:$A$2"),
        ],
        &[
            ("A1", "3"),
            ("A2", "4"),
            ("B1", "=Square(Cell)"),
            ("B2", "=SUM(Cells)"),
        ],
    );
    assert_eq!(number(&model, "Sheet1!B1"), 9.0);
    assert_eq!(number(&model, "Sheet1!B2"), 7.0);
}
