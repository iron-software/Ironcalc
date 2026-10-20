#![allow(clippy::unwrap_used)]

//! IronXL: the 1904 date system of a model.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

fn number(model: &Model, cell: &str) -> f64 {
    let reference = model._parse_reference(cell);
    match model
        .get_cell_value_by_index(reference.sheet, reference.row, reference.column)
        .unwrap()
    {
        CellValue::Number(value) => value,
        other => panic!("{cell} holds {other:?}"),
    }
}

fn model(date1904: bool) -> Model<'static> {
    let mut model = new_empty_model();
    model.set_date1904(date1904);
    model._set("C1", "0");
    model._set("A1", "=DATE(2026,1,1)");
    model._set("A2", "=YEAR(C1)");
    model._set("A3", "=MONTH(C1)*100+DAY(C1)");
    model._set("A4", "=DATE(1904,1,1)");
    model._set("A5", "=EDATE(C1,1)");
    model._set("A6", "=EOMONTH(C1,0)");
    model._set("A7", "=WEEKDAY(C1)");
    model._set("A8", "=DATEVALUE(\"2026-01-01\")");
    model._set("A9", "=DAYS360(C1,31)");
    model._set("A10", "=TEXT(C1,\"yyyy-mm-dd\")");
    model._set("A11", "=DATE(1903,12,31)");
    model._set("A12", "=YEAR(2957003)");
    model._set("A13", "=YEAR(2957004)");
    model._set("A14", "=WORKDAY(C1,1)");
    model.evaluate();
    model
}

#[test]
fn a_new_model_uses_the_1900_system() {
    let model = new_empty_model();
    assert!(!model.is_date1904());
}

#[test]
fn the_1904_system_gives_serial_0_to_1_january_1904() {
    let model = model(true);
    assert!(model.is_date1904());
    assert_eq!(number(&model, "A1"), 44561.0);
    assert_eq!(number(&model, "A2"), 1904.0);
    assert_eq!(number(&model, "A3"), 101.0);
    assert_eq!(number(&model, "A4"), 0.0);
    assert_eq!(number(&model, "A5"), 31.0);
    assert_eq!(number(&model, "A6"), 30.0);
    // 1 January 1904 is a Friday.
    assert_eq!(number(&model, "A7"), 6.0);
    assert_eq!(number(&model, "A8"), 44561.0);
    assert_eq!(number(&model, "A9"), 30.0);
    assert_eq!(model._get_text("A10"), "1904-01-01");
    assert_eq!(model._get_text("A11"), "#NUM!");
    assert_eq!(number(&model, "A12"), 9999.0);
    assert_eq!(model._get_text("A13"), "#NUM!");
    // The next workday after Friday 1 January 1904 is Monday 4 January 1904.
    assert_eq!(number(&model, "A14"), 3.0);
}

#[test]
fn the_1900_system_keeps_its_serials() {
    let model = model(false);
    assert_eq!(number(&model, "A1"), 46023.0);
    assert_eq!(number(&model, "A2"), 1900.0);
    assert_eq!(number(&model, "A4"), 1462.0);
    assert_eq!(number(&model, "A8"), 46023.0);
    assert_eq!(number(&model, "A3"), 100.0);
    assert_eq!(number(&model, "A12"), 9995.0);
}

#[test]
fn the_formatter_and_the_evaluation_of_given_cells_use_the_date_system() {
    let mut model = new_empty_model();
    model.set_date1904(true);
    model._set("B1", "=C1+44561");
    model.evaluate_cells(&[(0, 1, 2)]).unwrap();
    assert_eq!(number(&model, "B1"), 44561.0);
    model._set("B2", "=YEAR(B1)");
    model.evaluate_cells(&[(0, 2, 2)]).unwrap();
    assert_eq!(number(&model, "B2"), 2026.0);
    // The selection lasts only for the call: a 1900 model next to it keeps its own system.
    let mut other = new_empty_model();
    other._set("B2", "=YEAR(44561)");
    other.evaluate();
    assert_eq!(number(&other, "B2"), 2021.0);
}
