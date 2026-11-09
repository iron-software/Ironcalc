#![allow(clippy::unwrap_used)]

//! IronXL: a lookup range argument, the first argument of `INDEX`, and a criteria range argument
//! read a one-cell `INDEX` or `IF` result as a reference, as in Excel. A criteria range argument
//! also reads a direct one-cell reference as a range. `CELL` gives the error of an `INDEX` or
//! `IF` reference argument.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!B1` text, `B2` 5, and `B3` empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("E{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("E{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn a_lookup_range_reads_index_as_a_reference() {
    assert_results(&[
        ("=VLOOKUP(\"text\",INDEX(B1:B2,1),1,FALSE)", "text"),
        ("=HLOOKUP(\"text\",INDEX(B1:B2,1),1,FALSE)", "text"),
        ("=LOOKUP(\"text\",INDEX(B1:B2,1))", "text"),
        ("=LOOKUP(\"text\",INDEX(B1:B2,1),INDEX(B1:B2,1))", "text"),
        ("=XLOOKUP(\"text\",INDEX(B1:B2,1),INDEX(B1:B2,1))", "text"),
        ("=MATCH(\"text\",INDEX(B1:B2,1),0)", "1"),
        ("=XMATCH(\"text\",INDEX(B1:B2,1))", "1"),
        ("=VLOOKUP(\"text\",IF(TRUE,B1),1,FALSE)", "text"),
        ("=MATCH(5,IF(FALSE,B1,INDEX(B1:B2,2)),0)", "1"),
    ]);
}

#[test]
fn the_first_argument_of_index_reads_index_as_a_reference() {
    assert_results(&[
        ("=INDEX(INDEX(B1:B2,1),1)", "text"),
        ("=INDEX(INDEX(B1:B2,1),1,1)", "text"),
        ("=ROWS(INDEX(INDEX(B1:B2,1),1))", "1"),
        ("=INDEX(INDEX(B1:B2,1),2)", "#REF!"),
        ("=INDEX(IF(TRUE,B2),1)", "5"),
    ]);
}

#[test]
fn a_criteria_range_reads_a_one_cell_reference_as_a_range() {
    assert_results(&[
        ("=COUNTIF(B2,5)", "1"),
        ("=COUNTIF(INDEX(B1:B2,2),5)", "1"),
        ("=COUNTIF(IF(TRUE,B2),5)", "1"),
        ("=SUMIF(B2,5)", "5"),
        ("=SUMIF(INDEX(B1:B2,2),5)", "5"),
        ("=SUMIF(B1,\"text\",B2)", "5"),
        ("=SUMIF(INDEX(B1:B2,1),\"text\",INDEX(B1:B2,2))", "5"),
        ("=SUMIF(INDEX(B1:B2,2),{5,6})", "5"),
        ("=AVERAGEIF(INDEX(B1:B2,2),5)", "5"),
        ("=AVERAGEIF(B1,\"text\",B2)", "5"),
        ("=COUNTIFS(INDEX(B1:B2,2),5)", "1"),
        ("=COUNTIFS(B2,5,B1,\"text\")", "1"),
        ("=SUMIFS(INDEX(B1:B2,2),INDEX(B1:B2,2),5)", "5"),
        ("=SUMIFS(B2,B2,5)", "5"),
        ("=AVERAGEIFS(INDEX(B1:B2,2),INDEX(B1:B2,2),5)", "5"),
        ("=MINIFS(INDEX(B1:B2,2),INDEX(B1:B2,2),5)", "5"),
        ("=MAXIFS(INDEX(B1:B2,2),INDEX(B1:B2,2),5)", "5"),
    ]);
}

#[test]
fn cell_gives_the_error_of_an_index_reference() {
    assert_results(&[
        ("=CELL(\"address\",INDEX(B1:B2,3))", "#REF!"),
        ("=CELL(\"address\",IF(TRUE,INDEX(B1:B2,3)))", "#REF!"),
        ("=CELL(\"address\",5)", "#VALUE!"),
        ("=CELL(\"address\",INDEX(B1:B2,2))", "$B$2"),
        ("=CELL(\"address\",INDEX({1,2},1))", "#VALUE!"),
    ]);
}

#[test]
fn value_uses_ranges_and_errors_keep_their_results() {
    assert_results(&[
        ("=VLOOKUP(INDEX(B1:B2,1),B1:B2,1,FALSE)", "text"),
        ("=MATCH(INDEX(B1:B2,1),B1:B2,0)", "1"),
        ("=VLOOKUP(\"text\",INDEX(B1:C2,0,1),1,FALSE)", "text"),
        ("=VLOOKUP(\"text\",INDEX(B1:B2,3),1,FALSE)", "#REF!"),
        ("=COUNTIF(INDEX(B1:B2,3),5)", "#REF!"),
        ("=SUMIFS(INDEX(B1:B2,3),B2,5)", "#REF!"),
        ("=SUMIF(B2,5,INDEX(B1:B2,3))", "#REF!"),
        ("=VLOOKUP(\"text\",INDEX({1,2},1),1,FALSE)", "#N/A"),
        ("=COUNTIF(INDEX({1,2},1),5)", "#VALUE!"),
        ("=SUMIF(B1:B2,\"text\")", "0"),
        ("=VLOOKUP(\"text\",OFFSET(B1,0,0),1,FALSE)", "text"),
        ("=COUNTIF(OFFSET(B2,0,0),5)", "1"),
        ("=COUNTIF(B1:B2,5)", "1"),
    ]);
}

/// In each lookup and criteria context, `INDEX(B1:B3,k)` gives the result of the direct
/// reference `Bk`.
#[test]
fn index_gives_the_result_of_the_direct_reference() {
    let contexts = [
        "=VLOOKUP(\"text\",{},1,FALSE)",
        "=HLOOKUP(5,{},1,FALSE)",
        "=LOOKUP(\"text\",{})",
        "=XLOOKUP(5,{},{})",
        "=MATCH(5,{},0)",
        "=XMATCH(\"text\",{})",
        "=INDEX({},1)",
        "=COUNTIF({},5)",
        "=COUNTIF({},\"\")",
        "=SUMIF({},5)",
        "=SUMIF({},\"text\",B2)",
        "=AVERAGEIF({},5)",
        "=COUNTIFS({},5)",
        "=SUMIFS({},{},5)",
        "=AVERAGEIFS({},{},5)",
        "=MINIFS({},{},5)",
        "=MAXIFS({},{},5)",
    ];
    let mut model = model();
    let mut cases = Vec::new();
    for (index, context) in contexts.iter().enumerate() {
        for k in 1..=3 {
            let row = index * 3 + k;
            let direct = context.replace("{}", &format!("B{k}"));
            let index_form = context.replace("{}", &format!("INDEX(B1:B3,{k})"));
            model._set(&format!("E{row}"), &direct);
            model._set(&format!("F{row}"), &index_form);
            cases.push((row, direct, index_form));
        }
    }
    model.evaluate();
    for (row, direct, index_form) in cases {
        assert_eq!(
            model._get_text(&format!("F{row}")),
            model._get_text(&format!("E{row}")),
            "{index_form} and {direct}"
        );
    }
}

#[test]
fn a_criteria_result_follows_a_change_of_the_index_cell() {
    let mut model = model();
    model._set("E1", "=SUMIF(INDEX(B1:B2,2),\">0\")");
    model._set("E2", "=VLOOKUP(5,INDEX(B1:B2,2),1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "5");
    assert_eq!(model._get_text("E2"), "5");
    model._set("B2", "7");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "7");
    assert_eq!(model._get_text("E2"), "#N/A");
}
