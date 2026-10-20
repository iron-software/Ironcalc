#![allow(clippy::unwrap_used)]

// IronXL: the 1904 date system setting of the model.

use crate::cell::CellValue;
use crate::mock_time;
use crate::test::util::new_empty_model;

// 14:44 20 Mar 2023 Berlin
const TIMESTAMP_2023: i64 = 1679319865208;

fn number(model: &crate::model::Model, cell: &str) -> f64 {
    match model
        .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
        .unwrap()
    {
        CellValue::Number(f) => f,
        other => panic!("{cell} is not a number: {other:?}"),
    }
}

fn set_num_fmt(model: &mut crate::model::Model, row: i32, column: i32, num_fmt: &str) {
    let mut style = model.get_style_for_cell(0, row, column).unwrap();
    style.num_fmt = num_fmt.to_string();
    model.set_cell_style(0, row, column, &style).unwrap();
}

#[test]
fn default_is_1900() {
    let model = new_empty_model();
    assert!(!model.is_date_1904());
}

#[test]
fn set_and_reset_the_setting() {
    let mut model = new_empty_model();
    model._set("A1", "=DATE(2026,1,1)");
    model.evaluate();
    assert_eq!(number(&model, "A1"), 46023.0);

    model.set_date_1904(true);
    assert!(model.is_date_1904());
    assert_eq!(number(&model, "A1"), 44561.0);

    // Setting the same value again does not change the result.
    model.set_date_1904(true);
    assert_eq!(number(&model, "A1"), 44561.0);

    model.set_date_1904(false);
    assert!(!model.is_date_1904());
    assert_eq!(number(&model, "A1"), 46023.0);
}

#[test]
fn date_and_parts_1900() {
    let mut model = new_empty_model();
    // Serial 61 is 1900-03-01. The engine reads serials below 61 as dates from 1899-12-31.
    model._set("C1", "61");
    model._set("A1", "=DATE(2026,1,1)");
    model._set("A2", "=YEAR(C1)");
    model._set("A3", "=DATE(1904,1,1)");
    model._set("A4", "=YEAR(46023)");
    model.evaluate();

    assert_eq!(number(&model, "A1"), 46023.0);
    assert_eq!(number(&model, "A2"), 1900.0);
    assert_eq!(number(&model, "A3"), 1462.0);
    assert_eq!(number(&model, "A4"), 2026.0);
}

#[test]
fn date_and_parts_1904() {
    let mut model = new_empty_model();
    model.set_date_1904(true);
    model._set("C1", "0");
    model._set("A1", "=DATE(2026,1,1)");
    model._set("A2", "=YEAR(C1)");
    model._set("A3", "=DATE(1904,1,1)");
    model._set("A4", "=MONTH(C1)");
    model._set("A5", "=DAY(C1)");
    model._set("A6", "=YEAR(44561)");
    model._set("A7", "=DATE(1903,12,31)");
    model._set("A8", "=YEAR(-1)");
    model._set("A9", "=DATE(1904,3,1)");
    model.evaluate();

    assert_eq!(number(&model, "A1"), 44561.0);
    assert_eq!(number(&model, "A2"), 1904.0);
    assert_eq!(number(&model, "A3"), 0.0);
    assert_eq!(number(&model, "A4"), 1.0);
    assert_eq!(number(&model, "A5"), 1.0);
    assert_eq!(number(&model, "A6"), 2026.0);
    assert_eq!(model._get_text("A7"), "#NUM!");
    assert_eq!(model._get_text("A8"), "#NUM!");
    // 1904 is a leap year: 1904-02-29 is serial 59.
    assert_eq!(number(&model, "A9"), 60.0);
}

#[test]
fn date_functions_1904() {
    let mut model = new_empty_model();
    model.set_date_1904(true);
    model._set("A1", "=WEEKDAY(0)");
    model._set("A2", "=EDATE(0,1)");
    model._set("A3", "=EOMONTH(0,0)");
    model._set("A4", "=DATEVALUE(\"2026-01-01\")");
    model._set("A5", "=WEEKNUM(DATE(2026,1,1))");
    model._set("A6", "=ISOWEEKNUM(DATE(2026,1,1))");
    model._set("A7", "=WORKDAY(DATE(2026,1,2),1)=DATE(2026,1,5)");
    model._set("A8", "=NETWORKDAYS(DATE(2026,1,1),DATE(2026,1,31))");
    model._set("A9", "=DATEDIF(0,DATE(2026,1,1),\"Y\")");
    model._set("A10", "=YEARFRAC(DATE(2026,1,1),DATE(2026,7,1))");
    model._set("A11", "=DAYS(DATE(2026,1,1),0)");
    model._set("A12", "=DAYS360(0,DATE(1904,2,1))");
    model._set(
        "A13",
        "=COUPNCD(DATE(2026,1,15),DATE(2030,6,30),2)=DATE(2026,6,30)",
    );
    model._set(
        "A14",
        "=COUPPCD(DATE(2026,1,15),DATE(2030,6,30),2)=DATE(2025,12,31)",
    );
    model._set("A15", "=YEAR(DATE(9999,12,31))");
    model._set("A16", "=YEAR(DATE(9999,12,31)+1)");
    model._set("A17", "=EDATE(1,-1)");
    model._set("A18", "=COUNTIF(B1:B3,\">1/1/2026\")");
    model._set("B1", "=DATE(2025,12,31)");
    model._set("B2", "=DATE(2026,1,2)");
    model._set("B3", "=DATE(2026,1,3)");
    model.evaluate();

    // 1 January 1904 is a Friday.
    assert_eq!(number(&model, "A1"), 6.0);
    assert_eq!(number(&model, "A2"), 31.0);
    assert_eq!(number(&model, "A3"), 30.0);
    assert_eq!(number(&model, "A4"), 44561.0);
    assert_eq!(number(&model, "A5"), 1.0);
    assert_eq!(number(&model, "A6"), 1.0);
    assert_eq!(model._get_text("A7"), "TRUE");
    assert_eq!(number(&model, "A8"), 22.0);
    assert_eq!(number(&model, "A9"), 122.0);
    assert_eq!(number(&model, "A10"), 0.5);
    assert_eq!(number(&model, "A11"), 44561.0);
    assert_eq!(number(&model, "A12"), 30.0);
    assert_eq!(model._get_text("A13"), "TRUE");
    assert_eq!(model._get_text("A14"), "TRUE");
    assert_eq!(number(&model, "A15"), 9999.0);
    assert_eq!(model._get_text("A16"), "#NUM!");
    assert_eq!(model._get_text("A17"), "#NUM!");
    assert_eq!(number(&model, "A18"), 2.0);
}

#[test]
fn formatted_date_1900_and_1904() {
    let mut model = new_empty_model();
    model._set("A1", "44561");
    set_num_fmt(&mut model, 1, 1, "yyyy-mm-dd");
    model._set("A2", "=TEXT(0,\"yyyy-mm-dd\")");
    model._set("A3", "=TEXT(A1,\"yyyy-mm-dd\")");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2021-12-31");
    assert_eq!(model._get_text("A3"), "2021-12-31");

    model.set_date_1904(true);
    assert_eq!(model._get_text("A1"), "2026-01-01");
    assert_eq!(model._get_text("A2"), "1904-01-01");
    assert_eq!(model._get_text("A3"), "2026-01-01");
}

#[test]
fn typed_date_1904() {
    let mut model = new_empty_model();
    model.set_date_1904(true);
    model._set("A1", "2026-01-01");
    model._set("A2", "=A1+0");
    model._set("A3", "=\"2026-01-01\"+0");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "2026-01-01");
    assert_eq!(number(&model, "A2"), 44561.0);
    assert_eq!(number(&model, "A3"), 44561.0);
}

#[test]
fn today_and_now_1904() {
    mock_time::set_mock_time(TIMESTAMP_2023);
    let mut model = new_empty_model();
    model._set("A1", "=TODAY()");
    model._set("A2", "=TEXT(TODAY(),\"yyyy-mm-dd\")");
    model._set("B1", "=TODAY()");
    model._set("B2", "=NOW()");
    model.evaluate();
    let today_1900 = number(&model, "B1");
    let now_1900 = number(&model, "B2");
    assert_eq!(model._get_text("A2"), "2023-03-20");

    model.set_date_1904(true);
    let today_1904 = number(&model, "B1");
    let now_1904 = number(&model, "B2");
    assert_eq!(today_1900 - today_1904, 1462.0);
    assert!((now_1900 - now_1904 - 1462.0).abs() < 1e-6);
    assert_eq!(model._get_text("A2"), "2023-03-20");
}
