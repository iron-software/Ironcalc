#![allow(clippy::unwrap_used)]

use crate::test::util::new_empty_model;

// Cells are (sheet, row, column), 1-based rows and columns.
const A1: (u32, i32, i32) = (0, 1, 1);
const B1: (u32, i32, i32) = (0, 1, 2);
const C1: (u32, i32, i32) = (0, 1, 3);
const D1: (u32, i32, i32) = (0, 1, 4);

fn chain_model<'a>() -> crate::model::Model<'a> {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("B1", "=A1*2");
    model._set("C1", "=B1+1");
    model._set("D1", "=A1*10");
    model._set("E1", "=7");
    model.evaluate();
    model
}

#[test]
fn recalculates_the_given_cells_and_keeps_the_others() {
    let mut model = chain_model();
    model._set("A1", "5");
    model.evaluate_cells(&[B1, C1]).unwrap();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("C1"), "11");
    // D1 also depends on A1 but was not given, so it keeps its last result.
    assert_eq!(model._get_text("D1"), "10");
    assert_eq!(model._get_text("E1"), "7");
}

#[test]
fn the_order_of_the_given_cells_does_not_matter() {
    let mut model = chain_model();
    model._set("A1", "5");
    model.evaluate_cells(&[C1, B1]).unwrap();
    assert_eq!(model._get_text("B1"), "10");
    assert_eq!(model._get_text("C1"), "11");
}

#[test]
fn the_result_equals_whole_workbook_evaluation_when_every_stale_cell_is_given() {
    let mut partial = chain_model();
    let mut whole = chain_model();
    partial._set("A1", "3");
    whole._set("A1", "3");
    partial.evaluate_cells(&[B1, C1, D1]).unwrap();
    whole.evaluate();
    for cell in ["A1", "B1", "C1", "D1", "E1"] {
        assert_eq!(partial._get_text(cell), whole._get_text(cell), "{cell}");
    }
}

#[test]
fn a_new_formula_that_a_given_cell_reads_is_evaluated_too() {
    let mut model = chain_model();
    // F1 is new and was never evaluated; C1 now reads it.
    model._set("F1", "=A1+100");
    model._set("C1", "=F1+1");
    model.evaluate_cells(&[C1]).unwrap();
    assert_eq!(model._get_text("C1"), "102");
    assert_eq!(model._get_text("F1"), "101");
}

#[test]
fn works_on_a_model_that_was_never_evaluated() {
    let mut model = new_empty_model();
    model._set("A1", "4");
    model._set("B1", "=A1*2");
    model._set("C1", "=B1+1");
    model.evaluate_cells(&[C1]).unwrap();
    assert_eq!(model._get_text("C1"), "9");
}

#[test]
fn a_circular_reference_among_the_given_cells_is_an_error() {
    let mut model = new_empty_model();
    model._set("A1", "=B1");
    model._set("B1", "=A1");
    model.evaluate_cells(&[A1, B1]).unwrap();
    assert_eq!(model._get_text("A1"), "#CIRC!");
}

#[test]
fn an_empty_list_changes_nothing() {
    let mut model = chain_model();
    model._set("A1", "5");
    model.evaluate_cells(&[]).unwrap();
    assert_eq!(model._get_text("B1"), "2");
}

#[test]
fn repeated_calls_do_not_grow_the_support_of_a_cell() {
    use crate::expressions::types::CellReferenceIndex;
    let mut model = chain_model();
    let b1 = CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 2,
    };
    let after_evaluate = model.support.get(&b1).map_or(0, Vec::len);
    for value in 0..1000 {
        model._set("A1", &value.to_string());
        model.evaluate_cells(&[B1]).unwrap();
    }
    assert_eq!(model._get_text("B1"), "1998");
    assert_eq!(model.support.get(&b1).map_or(0, Vec::len), after_evaluate);
}

#[test]
fn an_unknown_sheet_index_is_an_error_and_changes_nothing() {
    let mut model = chain_model();
    model._set("A1", "5");
    let error = model.evaluate_cells(&[B1, (7, 1, 1)]).unwrap_err();
    assert_eq!(error, "Invalid sheet index: 7");
    // B1 was given too, but nothing was recalculated.
    assert_eq!(model._get_text("B1"), "2");
}
