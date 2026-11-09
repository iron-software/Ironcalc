#![allow(clippy::unwrap_used)]

// IronXL: the date functions read a 1900 serial before 1 March 1900 as Excel and IronXL ADR 029
// read it. Serial 1 is 1 January 1900, and serial 60 is the fictional 29 February 1900, which
// reads as 28 February 1900. The weekday is the weekday of Excel: serial 1 is a Sunday.

use crate::cell::CellValue;
use crate::model::Model;
use crate::test::util::new_empty_model;

fn evaluate(formulas: &[(&str, &str)], date_1904: bool) -> Model<'static> {
    let mut model = new_empty_model();
    model.set_date_1904(date_1904);
    for (cell, formula) in formulas {
        model._set(cell, formula);
    }
    model.evaluate();
    model
}

fn assert_values(formulas: &[(&str, &str, CellValue)], date_1904: bool) {
    let inputs: Vec<(&str, &str)> = formulas.iter().map(|(c, f, _)| (*c, *f)).collect();
    let model = evaluate(&inputs, date_1904);
    for (cell, formula, expected) in formulas {
        let actual = model
            .get_cell_value_by_ref(&format!("Sheet1!{cell}"))
            .unwrap();
        assert_eq!(&actual, expected, "{cell} {formula}");
    }
}

fn n(value: f64) -> CellValue {
    CellValue::Number(value)
}

fn s(value: &str) -> CellValue {
    CellValue::String(value.to_string())
}

#[test]
fn date_gives_the_excel_serial() {
    assert_values(
        &[
            ("A1", "=DATE(1900,1,1)", n(1.0)),
            ("A2", "=DATE(1900,1,31)", n(31.0)),
            ("A3", "=DATE(1900,2,28)", n(59.0)),
            ("A4", "=DATE(1900,3,1)", n(61.0)),
            ("A5", "=DATE(1900,2,29)", n(61.0)),
            ("A6", "=DATEVALUE(\"1900-01-01\")", n(1.0)),
            ("A7", "=DATE(2026,11,8)-DATE(1900,1,1)", n(46_333.0)),
            ("A8", "=DATE(2026,11,8)", n(46_334.0)),
            ("A9", "=DATE(1900,1,0)", s("#NUM!")),
            ("A10", "=DATEVALUE(\"2/28/1900\")", n(59.0)),
        ],
        false,
    );
}

#[test]
fn date_text_gives_the_excel_serial() {
    let mut model = new_empty_model();
    model._set("A1", "1900-01-01");
    model._set("A2", "1900-02-28");
    model._set("A3", "1900-03-01");
    model.evaluate();
    assert_eq!(model.get_cell_value_by_ref("Sheet1!A1"), Ok(n(1.0)));
    assert_eq!(model.get_cell_value_by_ref("Sheet1!A2"), Ok(n(59.0)));
    assert_eq!(model.get_cell_value_by_ref("Sheet1!A3"), Ok(n(61.0)));
    assert_eq!(model._get_text("A1"), "1900-01-01");
}

#[test]
fn date_parts_of_a_serial() {
    assert_values(
        &[
            ("A1", "=DAY(1)", n(1.0)),
            ("A2", "=MONTH(1)", n(1.0)),
            ("A3", "=YEAR(1)", n(1900.0)),
            ("A4", "=DAY(59)", n(28.0)),
            ("A5", "=MONTH(59)", n(2.0)),
            ("A6", "=DAY(60)", n(28.0)),
            ("A7", "=MONTH(60)", n(2.0)),
            ("A8", "=DAY(61)", n(1.0)),
            ("A9", "=MONTH(61)", n(3.0)),
            ("A10", "=EDATE(1,1)", n(32.0)),
            ("A11", "=EOMONTH(1,0)", n(31.0)),
            // The last day of February 1900 is the fictional 29 February, as in Excel.
            ("A12", "=EOMONTH(1,1)", n(60.0)),
            ("A13", "=DAY(0)", s("#NUM!")),
            ("A14", "=DATEDIF(1,61,\"d\")", n(60.0)),
            ("A15", "=DATEDIF(1,32,\"m\")", n(1.0)),
            ("A16", "=DATEDIF(1,367,\"y\")", n(1.0)),
            ("A17", "=DAYS360(1,61)", n(60.0)),
            ("A18", "=YEARFRAC(1,61,3)*365", n(60.0)),
            ("A19", "=YEAR(DATE(1900,1,1))", n(1900.0)),
        ],
        false,
    );
}

#[test]
fn day_counts_hold_the_fictional_29_february() {
    // Excel counts the fictional 29 February 1900 in a day count from a date before
    // 1 March 1900 (IronXL ADR 029).
    assert_values(
        &[
            ("A1", "=DATEDIF(1,61,\"yd\")", n(60.0)),
            ("A2", "=DATEDIF(2,61,\"yd\")", n(59.0)),
            ("A3", "=DATEDIF(1,60,\"yd\")", n(59.0)),
            ("A4", "=DATEDIF(1,59,\"yd\")", n(58.0)),
            (
                "A5",
                "=DATEDIF(DATE(2020,1,15),DATE(2026,3,1),\"yd\")",
                n(45.0),
            ),
            ("A6", "=DATEDIF(15,74,\"md\")", n(28.0)),
            ("A7", "=DATEDIF(15,58,\"md\")", n(12.0)),
            (
                "A8",
                "=DATEDIF(DATE(2026,1,15),DATE(2026,3,14),\"md\")",
                n(27.0),
            ),
            ("A9", "=ACCRINTM(1,61,0.36,1000,2)", n(60.0)),
            ("A10", "=ROUND(ACCRINTM(1,59,0.36,1000,2),9)", n(58.0)),
            ("A11", "=COUPDAYS(50,DATE(1900,8,15),2,1)", n(182.0)),
            (
                "A12",
                "=COUPDAYS(DATE(2026,2,19),DATE(2026,8,15),2,1)",
                n(181.0),
            ),
            ("A13", "=COUPDAYBS(50,DATE(1900,8,15),2,1)", n(4.0)),
            ("A14", "=COUPDAYSNC(50,DATE(1900,8,15),2,1)", n(178.0)),
        ],
        false,
    );
}

#[test]
fn weekday_of_excel() {
    assert_values(
        &[
            ("A1", "=WEEKDAY(1)", n(1.0)),
            ("A2", "=WEEKDAY(1,2)", n(7.0)),
            ("A3", "=WEEKDAY(59)", n(3.0)),
            ("A4", "=WEEKDAY(60)", n(4.0)),
            ("A5", "=WEEKDAY(61)", n(5.0)),
            ("A6", "=WEEKDAY(0)", s("#NUM!")),
            ("A7", "=WEEKNUM(7)", n(1.0)),
            ("A8", "=WEEKNUM(8)", n(2.0)),
            ("A9", "=WEEKNUM(1,2)", n(1.0)),
            ("A10", "=WEEKNUM(2,2)", n(2.0)),
            ("A11", "=WEEKNUM(1,21)", n(52.0)),
            ("A12", "=WEEKNUM(2,21)", n(1.0)),
            ("A13", "=ISOWEEKNUM(1)", n(52.0)),
            ("A14", "=ISOWEEKNUM(2)", n(1.0)),
            ("A15", "=ISOWEEKNUM(DATE(2026,11,8))", n(45.0)),
            ("A16", "=NETWORKDAYS(1,1)", n(0.0)),
            ("A17", "=NETWORKDAYS(1,7)", n(5.0)),
            ("A18", "=NETWORKDAYS.INTL(1,1)", n(0.0)),
            ("A19", "=NETWORKDAYS.INTL(1,7,11)", n(6.0)),
            ("A20", "=WORKDAY(6,1)", n(9.0)),
            ("A21", "=WORKDAY(9,-1)", n(6.0)),
            ("A22", "=WORKDAY(6,1,9)", n(10.0)),
            ("A23", "=WORKDAY(6,0)", n(6.0)),
            ("A24", "=WORKDAY.INTL(6,1)", n(9.0)),
            ("A25", "=WORKDAY.INTL(6,1,11)", n(7.0)),
            ("A26", "=WORKDAY.INTL(9,-1,1,6)", n(5.0)),
            ("A27", "=WORKDAY(59,1)", n(60.0)),
            ("A28", "=WORKDAY(DATE(2026,11,6),1)", n(46_334.0 + 1.0)),
            ("A29", "=TEXT(1,\"dddd\")", s("Sunday")),
            ("A30", "=TEXT(59,\"dddd\")", s("Tuesday")),
            ("A31", "=TEXT(61,\"dddd\")", s("Thursday")),
            ("A32", "=TEXT(0,\"ddd\")", s("Sat")),
            ("A33", "=TEXT(DATE(1900,1,1),\"m/d/yyyy\")", s("1/1/1900")),
            ("A34", "=TEXT(60,\"m/d/yyyy\")", s("2/28/1900")),
            ("A35", "=TEXT(0.5,\"d\")", s("31")),
        ],
        false,
    );
}

#[test]
fn date_1904_keeps_its_serials() {
    assert_values(
        &[
            ("A1", "=DATE(1904,1,1)", n(0.0)),
            ("A2", "=DAY(0)", n(1.0)),
            ("A3", "=YEAR(0)", n(1904.0)),
            ("A4", "=WEEKDAY(0)", n(6.0)),
            ("A5", "=DATE(2026,11,8)-DATE(1904,1,1)", n(44_872.0)),
            ("A6", "=ISOWEEKNUM(0)", n(53.0)),
            ("A7", "=WEEKNUM(0)", n(1.0)),
            ("A8", "=WEEKNUM(2)", n(2.0)),
            ("A9", "=WORKDAY(0,1)", n(3.0)),
            ("A10", "=NETWORKDAYS(0,6)", n(5.0)),
            ("A11", "=TEXT(0,\"dddd m/d/yyyy\")", s("Friday 1/1/1904")),
        ],
        true,
    );
}
