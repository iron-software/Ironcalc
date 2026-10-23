#![allow(clippy::unwrap_used)]

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

fn xlookup(formula: &str) -> String {
    let mut model = model_with_table();
    model._set("D1", formula);
    model.evaluate();
    model._get_text("D1")
}

// ── arrays as the lookup array and the return array ──────────────────────────

#[test]
fn row_array_constants() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2,3},{\"a\",\"b\",\"c\"})"), "b");
}

#[test]
fn column_array_constants() {
    assert_eq!(xlookup("=XLOOKUP(2,{1;2;3},{\"a\";\"b\";\"c\"})"), "b");
}

#[test]
fn array_result_of_a_function() {
    assert_eq!(xlookup("=XLOOKUP(2,SEQUENCE(3),{\"a\";\"b\";\"c\"})"), "b");
}

#[test]
fn range_lookup_array_and_array_return_array() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,{\"a\";\"b\";\"c\"})"), "b");
}

#[test]
fn array_lookup_array_and_range_return_array() {
    assert_eq!(xlookup("=XLOOKUP(2,{1;2;3},B1:B3)"), "b");
}

#[test]
fn row_array_and_row_range() {
    let mut model = new_empty_model();
    model._set("A1", "a");
    model._set("B1", "b");
    model._set("C1", "c");
    model._set("D1", "=XLOOKUP(3,{1,2,3},A1:C1)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "c");
}

#[test]
fn single_item_array() {
    assert_eq!(xlookup("=XLOOKUP(1,{1},{\"a\"})"), "a");
}

#[test]
fn error_item_of_the_return_array() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2},{\"a\",#DIV/0!})"), "#DIV/0!");
}

// ── not found, match modes, and search modes over arrays ─────────────────────

#[test]
fn not_found() {
    assert_eq!(xlookup("=XLOOKUP(9,{1,2,3},{\"a\",\"b\",\"c\"})"), "#N/A");
}

#[test]
fn if_not_found() {
    assert_eq!(
        xlookup("=XLOOKUP(9,{1,2,3},{\"a\",\"b\",\"c\"},\"none\")"),
        "none"
    );
}

#[test]
fn next_smaller() {
    assert_eq!(
        xlookup("=XLOOKUP(2.5,{1,2,3},{\"a\",\"b\",\"c\"},,-1)"),
        "b"
    );
}

#[test]
fn next_larger() {
    assert_eq!(xlookup("=XLOOKUP(2.5,{1,2,3},{\"a\",\"b\",\"c\"},,1)"), "c");
}

#[test]
fn wildcard() {
    assert_eq!(
        xlookup("=XLOOKUP(\"b*\",{\"apple\",\"banana\"},{1,2},,2)"),
        "2"
    );
}

#[test]
fn last_to_first() {
    assert_eq!(
        xlookup("=XLOOKUP(2,{1,2,2},{\"x\",\"y\",\"z\"},,0,-1)"),
        "z"
    );
}

#[test]
fn binary_search_ascending() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2,3},{\"a\",\"b\",\"c\"},,0,2)"), "b");
}

#[test]
fn binary_search_ascending_exact_not_found() {
    assert_eq!(
        xlookup("=XLOOKUP(2.5,{1,2,3},{\"a\",\"b\",\"c\"},,0,2)"),
        "#N/A"
    );
}

#[test]
fn binary_search_descending_next_larger() {
    assert_eq!(
        xlookup("=XLOOKUP(2.5,{3,2,1},{\"a\",\"b\",\"c\"},,1,-2)"),
        "a"
    );
}

#[test]
fn search_mode_zero_over_arrays() {
    assert_eq!(
        xlookup("=XLOOKUP(2,{1,2,3},{\"a\",\"b\",\"c\"},,0,0)"),
        "#VALUE!"
    );
}

#[test]
fn search_mode_zero_over_ranges() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,B1:B3,,0,0)"), "#VALUE!");
}

#[test]
fn search_mode_three_over_ranges() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,B1:B3,,0,3)"), "#VALUE!");
}

#[test]
fn binary_search_with_wildcard() {
    assert_eq!(
        xlookup("=XLOOKUP(2,{1,2,3},{\"a\",\"b\",\"c\"},,2,2)"),
        "#VALUE!"
    );
}

// ── shape errors ─────────────────────────────────────────────────────────────

#[test]
fn two_dimensional_lookup_array() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2;3,4},{\"a\",\"b\"})"), "#VALUE!");
}

#[test]
fn return_array_of_another_length() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2,3},{\"a\",\"b\"})"), "#VALUE!");
}

#[test]
fn return_array_of_another_orientation() {
    assert_eq!(
        xlookup("=XLOOKUP(2,{1,2,3},{\"a\";\"b\";\"c\"})"),
        "#VALUE!"
    );
}

#[test]
fn range_and_array_of_another_length() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,{\"a\";\"b\"})"), "#VALUE!");
}

#[test]
fn error_return_array() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2,3},1/0)"), "#DIV/0!");
}

#[test]
fn scalar_return_array() {
    assert_eq!(xlookup("=XLOOKUP(2,{1,2,3},5)"), "#VALUE!");
}

#[test]
fn scalar_lookup_array() {
    assert_eq!(xlookup("=XLOOKUP(2,2,{\"a\"})"), "#N/A");
}

// ── ranges keep their results ────────────────────────────────────────────────

#[test]
fn ranges() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,B1:B3)"), "b");
}

#[test]
fn ranges_binary_search() {
    assert_eq!(xlookup("=XLOOKUP(3,A1:A3,B1:B3,,0,2)"), "c");
}

#[test]
fn whole_column_ranges() {
    assert_eq!(xlookup("=XLOOKUP(3,A:A,B:B)"), "c");
}

#[test]
fn two_dimensional_range() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:B3,B1:B3)"), "#VALUE!");
}

#[test]
fn two_dimensional_whole_column_range() {
    assert_eq!(xlookup("=XLOOKUP(2,A:B,B:B)"), "#VALUE!");
}

#[test]
fn ranges_of_another_size() {
    assert_eq!(xlookup("=XLOOKUP(2,A1:A3,B1:B2)"), "#VALUE!");
}

// ── a one-cell reference is a vector of one item ─────────────────────────────

#[test]
fn one_cell_references() {
    assert_eq!(xlookup("=XLOOKUP(2,A2,B2)"), "b");
}

#[test]
fn one_cell_references_not_found() {
    assert_eq!(xlookup("=XLOOKUP(9,A2,B2)"), "#N/A");
}

#[test]
fn one_cell_references_if_not_found() {
    assert_eq!(xlookup("=XLOOKUP(9,A2,B2,\"none\")"), "none");
}

#[test]
fn absolute_one_cell_references() {
    assert_eq!(xlookup("=XLOOKUP(2,$A$2,$B$2)"), "b");
}

#[test]
fn one_cell_reference_and_single_item_array() {
    assert_eq!(xlookup("=XLOOKUP(2,A2,{\"only\"})"), "only");
}

#[test]
fn single_item_array_and_one_cell_reference() {
    assert_eq!(xlookup("=XLOOKUP(2,{2},B2)"), "b");
}

#[test]
fn one_cell_reference_and_range_of_another_size() {
    assert_eq!(xlookup("=XLOOKUP(2,A2,B1:B2)"), "#VALUE!");
}

#[test]
fn one_cell_reference_to_an_empty_cell() {
    assert_eq!(xlookup("=XLOOKUP(2,C2,B2)"), "#N/A");
}

#[test]
fn one_cell_references_to_another_sheet() {
    let mut model = model_with_table();
    model.new_sheet();
    model._set("Sheet2!A2", "2");
    model._set("Sheet2!B2", "other");
    model._set("D1", "=XLOOKUP(2,Sheet2!A2,Sheet2!B2)");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "other");
}

#[test]
fn one_cell_reference_follows_a_change_of_the_cell() {
    let mut model = model_with_table();
    model._set("D1", "=XLOOKUP(2,A2,B2)");
    model.evaluate();
    model._set("B2", "new");
    model.evaluate();
    assert_eq!(model._get_text("D1"), "new");
}

// D1 spills into D1:E1 and reads F2. F1 spills into F1:F2 after D1 in natural order.
// The one-cell reference F2 is a precedent of D1, so D1 moves after F1.
#[test]
fn one_cell_reference_into_a_later_spill_area() {
    let mut model = new_empty_model();
    model._set("D1", "=XLOOKUP(2,F2,{\"hit\"})&{\"1\",\"2\"}");
    model._set("F1", "=SEQUENCE(2)");
    model.evaluate();
    assert_eq!(model._get_text("F2"), "2");
    assert_eq!(model._get_text("D1"), "hit1");
    assert_eq!(model._get_text("E1"), "hit2");
}
