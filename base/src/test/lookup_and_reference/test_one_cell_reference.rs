#![allow(clippy::unwrap_used)]

// A one-cell reference is a range of one cell, as in Excel, for the range
// argument of MATCH, XMATCH, INDEX, LOOKUP, VLOOKUP, HLOOKUP, and XLOOKUP.

use crate::test::util::new_empty_model;

/// A model with A1:A3 = 1, 2, 3 and B1:B3 = a, b, c.
fn model_with_table() -> crate::Model<'static> {
    let mut model = new_empty_model();
    model._set("A1", "1");
    model._set("A2", "2");
    model._set("A3", "3");
    model._set("B1", "a");
    model._set("B2", "b");
    model._set("B3", "c");
    model
}

fn evaluate(formula: &str) -> String {
    let mut model = model_with_table();
    model._set("D1", formula);
    model.evaluate();
    model._get_text("D1")
}

// ── MATCH ────────────────────────────────────────────────────────────────────

#[test]
fn match_one_cell_reference() {
    assert_eq!(evaluate("=MATCH(2,A2,0)"), "1");
}

#[test]
fn match_one_cell_reference_sorted() {
    assert_eq!(evaluate("=MATCH(5,A2)"), "1");
    assert_eq!(evaluate("=MATCH(1,A2,-1)"), "1");
}

#[test]
fn match_absolute_one_cell_reference() {
    assert_eq!(evaluate("=MATCH(2,$A$2,0)"), "1");
}

#[test]
fn match_one_cell_reference_not_found() {
    assert_eq!(evaluate("=MATCH(9,A2,0)"), "#N/A");
}

#[test]
fn match_one_cell_reference_to_an_empty_cell() {
    assert_eq!(evaluate("=MATCH(2,C2,0)"), "#N/A");
}

#[test]
fn match_one_cell_reference_to_an_error() {
    let mut model = model_with_table();
    model._set("C2", "=1/0");
    model._set("D1", "=MATCH(2,C2,0)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
}

#[test]
fn match_one_cell_reference_to_another_sheet() {
    let mut model = model_with_table();
    model.new_sheet();
    model._set("Sheet2!A2", "2");
    model._set("D1", "=MATCH(2,Sheet2!A2,0)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
}

#[test]
fn match_range_keeps_its_result() {
    assert_eq!(evaluate("=MATCH(2,A1:A3,0)"), "2");
}

#[test]
fn match_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=MATCH(2,A2,0)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
    model._set("A2", "7");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    model._set("A2", "2");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
}

// ── XMATCH ───────────────────────────────────────────────────────────────────

#[test]
fn xmatch_one_cell_reference() {
    assert_eq!(evaluate("=XMATCH(2,A2)"), "1");
}

#[test]
fn xmatch_one_cell_reference_not_found() {
    assert_eq!(evaluate("=XMATCH(9,A2)"), "#N/A");
}

#[test]
fn xmatch_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=XMATCH(9,A2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    model._set("A2", "9");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "1");
}

// ── INDEX ────────────────────────────────────────────────────────────────────

#[test]
fn index_one_cell_reference() {
    assert_eq!(evaluate("=INDEX(B2,1)"), "b");
}

#[test]
fn index_one_cell_reference_row_and_column() {
    assert_eq!(evaluate("=INDEX(B2,1,1)"), "b");
}

#[test]
fn index_one_cell_reference_out_of_range() {
    assert_eq!(evaluate("=INDEX(B2,2)"), "#REF!");
    assert_eq!(evaluate("=INDEX(B2,1,2)"), "#REF!");
}

#[test]
fn index_range_keeps_its_result() {
    assert_eq!(evaluate("=INDEX(B1:B3,2)"), "b");
}

#[test]
fn index_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=INDEX(B2,1)");
    model.evaluate();
    model._set("B2", "new");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "new");
}

// ── LOOKUP ───────────────────────────────────────────────────────────────────

#[test]
fn lookup_one_cell_references() {
    assert_eq!(evaluate("=LOOKUP(2,A2,B2)"), "b");
}

#[test]
fn lookup_one_cell_reference_array_form() {
    assert_eq!(evaluate("=LOOKUP(2,A2)"), "2");
}

#[test]
fn lookup_one_cell_references_not_found() {
    assert_eq!(evaluate("=LOOKUP(1,A2,B2)"), "#N/A");
    assert_eq!(evaluate("=LOOKUP(1,A2)"), "#N/A");
}

// Not found in an ascending search is a value smaller than the first item.
// 9 is larger than 2, so the sorted search finds A2.
#[test]
fn lookup_one_cell_references_larger_value() {
    assert_eq!(evaluate("=LOOKUP(9,A2,B2)"), "b");
}

#[test]
fn lookup_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=LOOKUP(2,A2,B2)");
    model.evaluate();
    model._set("B2", "new");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "new");
}

// ── VLOOKUP ──────────────────────────────────────────────────────────────────

#[test]
fn vlookup_one_cell_reference() {
    assert_eq!(evaluate("=VLOOKUP(2,A2,1,FALSE)"), "2");
}

#[test]
fn vlookup_one_cell_reference_not_found() {
    assert_eq!(evaluate("=VLOOKUP(9,A2,1,FALSE)"), "#N/A");
}

#[test]
fn vlookup_one_cell_reference_column_out_of_range() {
    assert_eq!(evaluate("=VLOOKUP(2,A2,2,FALSE)"), "#REF!");
}

#[test]
fn vlookup_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=VLOOKUP(2,A2,1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    model._set("A2", "9");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    model._set("A2", "2");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
}

// ── HLOOKUP ──────────────────────────────────────────────────────────────────

#[test]
fn hlookup_one_cell_reference() {
    assert_eq!(evaluate("=HLOOKUP(2,A2,1,FALSE)"), "2");
}

#[test]
fn hlookup_one_cell_reference_not_found() {
    assert_eq!(evaluate("=HLOOKUP(9,A2,1,FALSE)"), "#N/A");
}

#[test]
fn hlookup_one_cell_reference_row_out_of_range() {
    assert_eq!(evaluate("=HLOOKUP(2,A2,2,FALSE)"), "#REF!");
}

#[test]
fn hlookup_one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=HLOOKUP(2,A2,1,FALSE)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
    model._set("A2", "9");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "#N/A");
    model._set("A2", "2");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "2");
}

// ── XLOOKUP ──────────────────────────────────────────────────────────────────

#[test]
fn xlookup_one_cell_references_keep_their_result() {
    assert_eq!(evaluate("=XLOOKUP(2,A2,B2)"), "b");
}
