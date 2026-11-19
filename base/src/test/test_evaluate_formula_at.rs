#![allow(clippy::unwrap_used)]

use crate::cell::CellValue;
use crate::expressions::token::Error;
use crate::expressions::types::{Area, CellReferenceIndex};
use crate::test::util::new_empty_model;
use crate::types::Cell;
use crate::FormulaResult;

fn at(row: i32, column: i32) -> CellReferenceIndex {
    CellReferenceIndex {
        sheet: 0,
        row,
        column,
    }
}

const A2: (i32, i32) = (2, 1);

fn a2() -> CellReferenceIndex {
    at(A2.0, A2.1)
}

/// A2:A5 = 1, 2, 2, 3. B1 = A2*10 and B2 = SUM(A2:A5).
fn model<'a>() -> crate::model::Model<'a> {
    let mut model = new_empty_model();
    model._set("A2", "1");
    model._set("A3", "2");
    model._set("A4", "2");
    model._set("A5", "3");
    model._set("B1", "=A2*10");
    model._set("B2", "=SUM(A2:A5)");
    model.evaluate();
    model
}

#[test]
fn a_relative_reference_moves_from_the_anchor_to_the_cell() {
    let mut model = model();
    let formula = "=COUNTIF($A:$A,A2)=1";
    assert_eq!(
        model
            .evaluate_formula_at(formula, a2(), a2(), None)
            .unwrap(),
        FormulaResult::Boolean(true)
    );
    assert_eq!(
        model
            .evaluate_formula_at(formula, a2(), at(3, 1), None)
            .unwrap(),
        FormulaResult::Boolean(false)
    );
    assert_eq!(
        model
            .evaluate_formula_at(formula, a2(), at(5, 1), None)
            .unwrap(),
        FormulaResult::Boolean(true)
    );
}

#[test]
fn the_leading_equals_sign_is_optional() {
    let mut model = model();
    assert_eq!(
        model.evaluate_formula_at("A2+1", a2(), a2(), None).unwrap(),
        FormulaResult::Number(2.0)
    );
}

#[test]
fn a_test_value_replaces_the_cell_during_the_evaluation_only() {
    let mut model = model();
    let formula = "=COUNTIF($A:$A,A2)=1";
    let two = CellValue::Number(2.0);
    assert_eq!(
        model
            .evaluate_formula_at(formula, a2(), a2(), Some(&two))
            .unwrap(),
        FormulaResult::Boolean(false)
    );
    assert_eq!(model._get_text("A2"), "1");
    let text = CellValue::String("new text".to_string());
    assert_eq!(
        model
            .evaluate_formula_at("=LEN(A2)", a2(), a2(), Some(&text))
            .unwrap(),
        FormulaResult::Number(8.0)
    );
    assert_eq!(
        model
            .evaluate_formula_at("=A2", a2(), a2(), Some(&CellValue::Boolean(true)))
            .unwrap(),
        FormulaResult::Boolean(true)
    );
    assert_eq!(
        model
            .evaluate_formula_at("=ISBLANK(A2)", a2(), a2(), Some(&CellValue::None))
            .unwrap(),
        FormulaResult::Boolean(true)
    );
    assert_eq!(model._get_text("A2"), "1");
}

#[test]
fn a_test_value_in_an_empty_cell_leaves_the_cell_empty() {
    let mut model = model();
    let c9 = at(9, 3);
    let value = CellValue::String("x".to_string());
    assert_eq!(
        model
            .evaluate_formula_at("=C9", c9, c9, Some(&value))
            .unwrap(),
        FormulaResult::Text("x".to_string())
    );
    assert!(!model.workbook.worksheets[0].sheet_data.contains_key(&9));
    let strings = model.workbook.shared_strings.clone();
    assert!(!strings.contains(&"x".to_string()));
    assert!(!model.shared_strings.contains_key("x"));
}

#[test]
fn a_test_text_that_the_workbook_holds_keeps_its_shared_string() {
    let mut model = model();
    model._set("D1", "same");
    model.evaluate();
    let count = model.workbook.shared_strings.len();
    let value = CellValue::String("same".to_string());
    assert_eq!(
        model
            .evaluate_formula_at("=A2", a2(), a2(), Some(&value))
            .unwrap(),
        FormulaResult::Text("same".to_string())
    );
    assert_eq!(model.workbook.shared_strings.len(), count);
    assert!(model.shared_strings.contains_key("same"));
}

#[test]
fn a_formula_of_another_cell_keeps_its_result() {
    let mut model = model();
    let value = CellValue::Number(100.0);
    // B1 reads A2, but it keeps its current result 10 during the evaluation.
    assert_eq!(
        model
            .evaluate_formula_at("=B1+A2", a2(), a2(), Some(&value))
            .unwrap(),
        FormulaResult::Number(110.0)
    );
    assert_eq!(model._get_text("B1"), "10");
}

#[test]
fn a_formula_without_a_result_is_not_evaluated() {
    let mut model = model();
    model._set("C1", "=A2+5");
    // No evaluation gave C1 a result, and the call does not give it one.
    let before = model.workbook.worksheets[0].cell(1, 3).cloned();
    let result = model.evaluate_formula_at("=C1", a2(), a2(), None).unwrap();
    assert!(matches!(result, FormulaResult::Error(Error::ERROR)));
    assert_eq!(model.workbook.worksheets[0].cell(1, 3).cloned(), before);
    assert!(!model.cells.contains_key(&(0, 1, 3)));
}

#[test]
fn the_model_is_the_same_after_the_call() {
    let mut model = model();
    let sheet_data = model.workbook.worksheets[0].sheet_data.clone();
    let states = model.cells.len();
    let support: std::collections::HashMap<_, _> = model
        .support
        .iter()
        .map(|(cell, cells)| (*cell, cells.len()))
        .collect();
    for value in [
        None,
        Some(CellValue::Number(9.0)),
        Some(CellValue::String("t".to_string())),
    ] {
        model
            .evaluate_formula_at("=SUM(A2:A5)+B2+LET(x,1,x)", a2(), at(2, 2), value.as_ref())
            .unwrap();
    }
    assert_eq!(model.workbook.worksheets[0].sheet_data, sheet_data);
    assert_eq!(model.cells.len(), states);
    let after: std::collections::HashMap<_, _> = model
        .support
        .iter()
        .map(|(cell, cells)| (*cell, cells.len()))
        .collect();
    assert_eq!(after, support);
    assert!(support.contains_key(&at(2, 2)));
    assert!(!model.read_only);
    model._set("A2", "4");
    model.evaluate();
    assert_eq!(model._get_text("B2"), "11");
}

#[test]
fn a_reference_gives_its_area() {
    let mut model = model();
    assert_eq!(
        model
            .evaluate_formula_at("=$A$5:$A$2", a2(), a2(), None)
            .unwrap(),
        FormulaResult::Reference(Area {
            sheet: 0,
            row: 2,
            column: 1,
            width: 1,
            height: 4
        })
    );
    model.add_defined_names(&[("Items", None, "Sheet1!$A$2:$B$3")]);
    assert_eq!(
        model
            .evaluate_formula_at("=Items", a2(), a2(), None)
            .unwrap(),
        FormulaResult::Reference(Area {
            sheet: 0,
            row: 2,
            column: 1,
            width: 2,
            height: 2
        })
    );
}

#[test]
fn each_kind_of_result_has_its_form() {
    let mut model = model();
    let mut result = |formula: &str| {
        model
            .evaluate_formula_at(formula, a2(), a2(), None)
            .unwrap()
    };
    assert_eq!(
        result("=\"a\"&\"b\""),
        FormulaResult::Text("ab".to_string())
    );
    assert_eq!(result("=1/0"), FormulaResult::Error(Error::DIV));
    assert_eq!(result("=Z99"), FormulaResult::Empty);
    assert_eq!(result("={4,5}"), FormulaResult::Number(4.0));
    assert_eq!(result("={\"x\",5}"), FormulaResult::Text("x".to_string()));
    assert_eq!(result("={TRUE,5}"), FormulaResult::Boolean(true));
    assert_eq!(result("={#N/A,5}"), FormulaResult::Error(Error::NA));
    assert_eq!(result("=LAMBDA(x,x)"), FormulaResult::Error(Error::CALC));
    assert!(matches!(result("=SUM("), FormulaResult::Error(_)));
}

#[test]
fn an_unknown_sheet_is_an_error_and_changes_nothing() {
    let mut model = model();
    let sheet_data = model.workbook.worksheets[0].sheet_data.clone();
    let other = CellReferenceIndex {
        sheet: 3,
        row: 1,
        column: 1,
    };
    assert_eq!(
        model.evaluate_formula_at("=1", other, a2(), Some(&CellValue::Number(1.0))),
        Err("Invalid sheet index: 3".to_string())
    );
    assert_eq!(
        model.evaluate_formula_at("=1", a2(), other, None),
        Err("Invalid sheet index: 3".to_string())
    );
    assert_eq!(model.workbook.worksheets[0].sheet_data, sheet_data);
}

#[test]
fn a_spill_cell_gives_its_current_value() {
    let mut model = new_empty_model();
    model._set("A1", "=SEQUENCE(3)");
    model.evaluate();
    let a1 = at(1, 1);
    assert_eq!(
        model.evaluate_formula_at("=A3", a1, a1, None).unwrap(),
        FormulaResult::Number(3.0)
    );
    assert!(matches!(
        model.workbook.worksheets[0].cell(3, 1),
        Some(Cell::SpillCell { .. })
    ));
}
