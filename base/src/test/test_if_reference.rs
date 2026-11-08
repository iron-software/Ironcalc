#![allow(clippy::unwrap_used)]

//! IronXL: `IF` with a scalar condition gives the reference of its chosen argument, as in Excel.
//! A function that reads a reference argument reads `IF(TRUE,B1)` as it reads `B1`. A use that
//! needs a value reads the value of the cell. An array condition gives an array.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!B1` text, `B2` 5, `B3` TRUE, `B4` FALSE, `C1` `#DIV/0!`, and `B5:B6`
/// empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("B3", "TRUE");
    model._set("B4", "FALSE");
    model._set("C1", "=1/0");
    model
}

/// The text of each formula, evaluated in one model.
fn results(formulas: &[&str]) -> Vec<String> {
    let mut model = model();
    for (index, formula) in formulas.iter().enumerate() {
        model._set(&format!("E{}", index + 1), formula);
    }
    model.evaluate();
    (0..formulas.len())
        .map(|index| model._get_text(&format!("E{}", index + 1)))
        .collect()
}

/// Checks that each formula gives its expected text.
fn assert_results(cases: &[(&str, &str)]) {
    let formulas: Vec<&str> = cases.iter().map(|(formula, _)| *formula).collect();
    let actual = results(&formulas);
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(actual[index], *expected, "{formula}");
    }
}

#[test]
fn a_reference_argument_reads_if_as_a_reference() {
    assert_results(&[
        ("=MINA(IF(TRUE,B1),B2)", "0"),
        ("=VARA(IF(TRUE,B1),B2)", "12.5"),
        ("=SUM(IF(TRUE,B1),B2)", "5"),
        ("=AVERAGE(IF(TRUE,B1),B2)", "5"),
        ("=SUM(IF(TRUE,B3),B2)", "5"),
        ("=COUNT(IF(TRUE,B3))", "0"),
        ("=COUNTA(IF(TRUE,B1),B2)", "2"),
        ("=MINA(IF(FALSE,1,B1),B2)", "0"),
        ("=MINA(IF(B3,B1),B2)", "0"),
        ("=MINA(IF(TRUE,B5),B2)", "5"),
        ("=MAXA(IF(TRUE,B4),B2)", "5"),
        ("=AND(IF(TRUE,B1),B3)", "TRUE"),
        ("=ISREF(IF(TRUE,B1))", "TRUE"),
        ("=ISREF(IF(FALSE,1,B1))", "TRUE"),
        ("=ISREF(IF(TRUE,B1:B2))", "TRUE"),
    ]);
}

#[test]
fn the_other_reference_uses_of_if_read_a_reference() {
    assert_results(&[
        ("=ROW(IF(TRUE,B2))", "2"),
        ("=COLUMN(IF(TRUE,B2))", "2"),
        ("=ROWS(IF(TRUE,B2))", "1"),
        ("=CELL(\"address\",IF(TRUE,B2))", "$B$2"),
        ("=ISFORMULA(IF(TRUE,C1))", "TRUE"),
        ("=FORMULATEXT(IF(TRUE,C1))", "=1/0"),
        ("=SUM(IF(TRUE,B1):B2)", "5"),
        ("=SUM(B1:IF(TRUE,B2))", "5"),
    ]);
}

#[test]
fn a_nested_if_or_index_in_if_gives_a_reference() {
    assert_results(&[
        ("=ISREF(IF(TRUE,INDEX(B1:B2,1)))", "TRUE"),
        ("=MINA(IF(TRUE,INDEX(B1:B2,1)),B2)", "0"),
        ("=MINA(IF(TRUE,IF(TRUE,B1)),B2)", "0"),
        ("=ISREF(IF(TRUE,IF(FALSE,1,B1)))", "TRUE"),
        ("=MINA(INDEX(IF(TRUE,B1:B2),1),B2)", "0"),
    ]);
}

#[test]
fn a_value_use_of_if_reads_the_value_of_the_cell() {
    assert_results(&[
        ("=IF(TRUE,B2)+1", "6"),
        ("=LEN(IF(TRUE,B1))", "4"),
        ("=IF(TRUE,B1)", "text"),
        ("=ISTEXT(IF(TRUE,B1))", "TRUE"),
        ("=TYPE(IF(TRUE,B1))", "2"),
        ("=IF(TRUE,B1)&\"x\"", "textx"),
        ("=N(IF(TRUE,B1))", "0"),
        ("=VLOOKUP(IF(TRUE,B1),B1:B2,1,FALSE)", "text"),
        ("=IF(FALSE,B1)", "FALSE"),
    ]);
}

#[test]
fn an_if_that_gives_a_value_or_an_error_keeps_its_result() {
    assert_results(&[
        ("=ISREF(IF(FALSE,B1))", "FALSE"),
        ("=ISREF(IF(TRUE,1,B1))", "FALSE"),
        ("=SUM(IF(TRUE,1,B1),B2)", "6"),
        ("=ISREF(IF(C1,B1))", "FALSE"),
        ("=MINA(IF(C1,B1),B2)", "#DIV/0!"),
        ("=MINA(IF(B1,B1),B2)", "#VALUE!"),
        ("=SUM(IF(TRUE,C1),B2)", "#DIV/0!"),
        ("=ISREF(IF(TRUE))", "FALSE"),
        ("=SUM(IF(TRUE),B2)", "#ERROR!"),
        ("=ROW(IF(TRUE,1))", "#VALUE!"),
    ]);
}

#[test]
fn an_if_with_an_array_condition_keeps_its_array() {
    assert_results(&[
        ("=ISREF(IF({TRUE,FALSE},B1,B2))", "FALSE"),
        ("=SUM(IF({TRUE,FALSE},B2,B2))", "10"),
        ("=SUM(IF(B3:B4,B2,1))", "6"),
        ("=ISREF(IF(B3:B4,B2,1))", "FALSE"),
    ]);
}

/// In each context, `IF(TRUE,B1)` and `IF(FALSE,1,B1)` give the result of `B1`.
#[test]
fn if_gives_the_result_of_the_direct_reference() {
    let contexts = [
        "=SUM({},B2)",
        "=AVERAGE({},B2)",
        "=COUNT({},B2)",
        "=COUNTA({},B2)",
        "=PRODUCT({},B2)",
        "=MINA({},B2)",
        "=VARA({},B2)",
        "=AND({},B3)",
        "=ISREF({})",
        "=ROW({})",
        "=COLUMN({})",
        "=ROWS({})",
        "=CELL(\"address\",{})",
        "=ISFORMULA({})",
        "=SUM({}:B2)",
        "={}+1",
        "={}&\"x\"",
        "=LEN({})",
        "=UPPER({})",
        "=ISTEXT({})",
        "=VLOOKUP({},B1:B2,1,FALSE)",
        "=MATCH({},B1:B2,0)",
        "=TYPE({})",
        "=N({})",
        "=EXACT({},\"text\")",
    ];
    let direct_formulas: Vec<String> = contexts.iter().map(|c| c.replace("{}", "B1")).collect();
    let direct_refs: Vec<&str> = direct_formulas.iter().map(String::as_str).collect();
    let direct_results = results(&direct_refs);
    for if_text in ["IF(TRUE,B1)", "IF(FALSE,1,B1)"] {
        let if_formulas: Vec<String> = contexts.iter().map(|c| c.replace("{}", if_text)).collect();
        let if_refs: Vec<&str> = if_formulas.iter().map(String::as_str).collect();
        let if_results = results(&if_refs);
        for (position, formula) in if_formulas.iter().enumerate() {
            assert_eq!(
                if_results[position], direct_results[position],
                "{formula} and {}",
                direct_formulas[position]
            );
        }
    }
}

#[test]
fn if_with_a_reference_follows_a_change_of_its_cell() {
    let mut model = model();
    model._set("E1", "=MINA(IF(TRUE,B1),B2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "0");
    model._set("B1", "3");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "3");
}

#[test]
fn if_of_a_reference_on_another_sheet_gives_the_value_of_its_cell() {
    let mut model = model();
    model.add_sheet("Sheet2").unwrap();
    model._set("Sheet2!A1", "=IF(TRUE,Sheet1!B1)");
    model._set("Sheet2!A2", "=IF(TRUE,Sheet1!$B$2)+1");
    model._set("Sheet2!A3", "=MINA(IF(TRUE,Sheet1!B1),Sheet1!B2)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!A1"), "text");
    assert_eq!(model._get_text("Sheet2!A2"), "6");
    assert_eq!(model._get_text("Sheet2!A3"), "0");
}
