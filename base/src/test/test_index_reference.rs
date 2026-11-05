#![allow(clippy::unwrap_used)]

//! IronXL: `INDEX` with a reference gives a reference, as in Excel. A function that reads a
//! reference argument reads `INDEX(B1:B2,k)` as it reads the direct reference to row k. A use
//! that needs a value reads the value of the cell. The array form gives a value.

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
fn a_reference_argument_reads_index_as_a_reference() {
    assert_results(&[
        ("=SUM(INDEX(B1:B2,1),B2)", "5"),
        ("=AVERAGE(INDEX(B1:B2,1),B2)", "5"),
        ("=COUNT(INDEX(B1:B2,1),B2)", "1"),
        ("=COUNTA(INDEX(B5:B6,1),B2)", "1"),
        ("=PRODUCT(INDEX(B1:B2,1),B2)", "5"),
        ("=MINA(INDEX(B1:B2,1),B2)", "0"),
        ("=VARA(INDEX(B1:B2,1),B2)", "12.5"),
        ("=MAXA(INDEX(B3:B4,1),B2)", "5"),
        ("=AND(INDEX(B1:B3,1),B3)", "TRUE"),
        ("=OR(INDEX(B1:B4,1),B4)", "FALSE"),
        ("=ISREF(INDEX(B1:B2,1))", "TRUE"),
        ("=ISREF(INDEX(B1:B2,0))", "TRUE"),
        ("=MINA(OFFSET(B1,0,0),B2)", "0"),
        ("=MINA(INDIRECT(\"B1\"),B2)", "0"),
    ]);
}

#[test]
fn the_other_reference_uses_of_index_read_a_reference() {
    assert_results(&[
        ("=ROW(INDEX(B1:B2,2))", "2"),
        ("=COLUMN(INDEX(B1:B2,1))", "2"),
        ("=ROWS(INDEX(B1:B2,1))", "1"),
        ("=CELL(\"address\",INDEX(B1:B2,2))", "$B$2"),
        ("=ISFORMULA(INDEX(B1:B2,1))", "FALSE"),
        ("=ISFORMULA(INDEX(C1:C2,1))", "TRUE"),
        ("=FORMULATEXT(INDEX(C1:C2,1))", "=1/0"),
        ("=SUM(INDEX(B1:B2,1):B2)", "5"),
        ("=SUM(B1:INDEX(B1:B2,2))", "5"),
    ]);
}

#[test]
fn a_value_use_of_index_reads_the_value_of_the_cell() {
    assert_results(&[
        ("=INDEX(B1:B2,2)", "5"),
        ("=INDEX(B1:B2,2)+1", "6"),
        ("=INDEX(B1:B2,1)&\"x\"", "textx"),
        ("=LEN(INDEX(B1:B2,1))", "4"),
        ("=UPPER(INDEX(B1:B2,1))", "TEXT"),
        ("=ISTEXT(INDEX(B1:B2,1))", "TRUE"),
        ("=VLOOKUP(INDEX(B1:B2,1),B1:B2,1,FALSE)", "text"),
        ("=MATCH(INDEX(B1:B2,1),B1:B2,0)", "1"),
        ("=TYPE(INDEX(B1:B2,1))", "2"),
        ("=N(INDEX(B1:B2,1))", "0"),
        ("=EXACT(INDEX(B1:B2,1),\"text\")", "TRUE"),
    ]);
}

#[test]
fn the_array_form_and_the_errors_of_index_keep_their_results() {
    assert_results(&[
        ("=ISREF(INDEX({1,2},1))", "FALSE"),
        ("=INDEX({1,2},1,2)", "2"),
        ("=SUM(INDEX({1,2},1,2),B2)", "7"),
        ("=INDEX(B1:B2,3)", "#REF!"),
        ("=SUM(INDEX(B1:B2,3),B2)", "#REF!"),
        ("=ISREF(INDEX(B1:B2,3))", "FALSE"),
        ("=ROW(INDEX(B1:B2,3))", "#REF!"),
        ("=INDEX(B1:B2,1,1,2)", "#REF!"),
        ("=SUM(INDEX(C1:C2,1),B2)", "#DIV/0!"),
        ("=INDEX(B5:B6,1)", "0"),
        ("=SUM(INDEX(B5:B6,1),B2)", "5"),
    ]);
}

/// In each context, `INDEX(B1:B2,k)` gives the result of the direct reference to row k.
#[test]
fn index_gives_the_result_of_the_direct_reference() {
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
    for (row, direct) in [(1, "B1"), (2, "B2")] {
        let index = format!("INDEX(B1:B2,{row})");
        let direct_formulas: Vec<String> =
            contexts.iter().map(|c| c.replace("{}", direct)).collect();
        let index_formulas: Vec<String> =
            contexts.iter().map(|c| c.replace("{}", &index)).collect();
        let direct_refs: Vec<&str> = direct_formulas.iter().map(String::as_str).collect();
        let index_refs: Vec<&str> = index_formulas.iter().map(String::as_str).collect();
        let direct_results = results(&direct_refs);
        let index_results = results(&index_refs);
        for (position, formula) in index_formulas.iter().enumerate() {
            assert_eq!(
                index_results[position], direct_results[position],
                "{formula} and {}",
                direct_formulas[position]
            );
        }
    }
}

#[test]
fn index_with_a_reference_follows_a_change_of_its_cell() {
    let mut model = model();
    model._set("E1", "=SUM(INDEX(B1:B2,1),B2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "5");
    model._set("B1", "3");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "8");
}

#[test]
fn index_of_a_range_on_another_sheet_gives_the_value_of_its_cell() {
    let mut model = model();
    model.add_sheet("Sheet2").unwrap();
    model._set("Sheet2!A1", "=INDEX(Sheet1!B1:B2,1)");
    model._set("Sheet2!A2", "=INDEX(Sheet1!$B$1:$B$2,2)+1");
    model._set("Sheet2!A3", "=SUM(INDEX(Sheet1!B1:B2,1),Sheet1!B2)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet2!A1"), "text");
    assert_eq!(model._get_text("Sheet2!A2"), "6");
    assert_eq!(model._get_text("Sheet2!A3"), "5");
}
