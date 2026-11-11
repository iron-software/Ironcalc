#![allow(clippy::unwrap_used)]

//! IronXL: `ISNUMBER` of an array or a range gives one Boolean for each element, in the shape of
//! the argument, as in Excel. Only a number element gives TRUE. A scalar keeps its result.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!L1:L4` = 2, 1, 2, 1 and `G1:G4` = a, 1, TRUE, empty. Column A is empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    for (index, number) in ["2", "1", "2", "1"].iter().enumerate() {
        model._set(&format!("L{}", index + 1), number);
    }
    model._set("G1", "a");
    model._set("G2", "1");
    model._set("G3", "TRUE");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model. Each formula gives
/// one value, so no result spills into the next formula.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("Z{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("Z{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn isnumber_of_an_array_or_a_range_counts_the_numbers() {
    assert_results(&[
        ("=SUMPRODUCT(--ISNUMBER({1,\"a\",3}))", "2"),
        ("=SUMPRODUCT(ISNUMBER({1,\"a\",3})*1)", "2"),
        ("=INDEX(ISNUMBER({1,\"a\",3}),1,3)", "TRUE"),
        ("=SUMPRODUCT(--ISNUMBER(L1:L4))", "4"),
        ("=SUMPRODUCT(ISNUMBER(L1:L4)*1)", "4"),
    ]);
}

#[test]
fn only_a_number_element_gives_true() {
    assert_results(&[
        ("=INDEX(ISNUMBER({1;\"a\"}),1,1)", "TRUE"),
        ("=INDEX(ISNUMBER({1;\"a\"}),2,1)", "FALSE"),
        ("=SUMPRODUCT(--ISNUMBER({1,\"2\",TRUE,#N/A}))", "1"),
        ("=INDEX(ISNUMBER({1,\"2\",TRUE,#N/A}),1,4)", "FALSE"),
        ("=SUMPRODUCT(--ISNUMBER(G1:G4))", "1"),
        ("=INDEX(ISNUMBER(G1:G4),2,1)", "TRUE"),
        ("=INDEX(ISNUMBER(G1:G4),4,1)", "FALSE"),
        ("=SUMPRODUCT(--ISNUMBER(A1:A2))", "0"),
        ("=SUMPRODUCT(--ISNUMBER(1/{1,0}))", "1"),
    ]);
}

#[test]
fn isnumber_keeps_the_shape_of_its_argument() {
    assert_results(&[
        ("=ROWS(ISNUMBER(L1:L4))", "4"),
        ("=COLUMNS(ISNUMBER(L1:L4))", "1"),
        ("=ROWS(ISNUMBER({1,2,3}))", "1"),
        ("=COLUMNS(ISNUMBER({1,2,3}))", "3"),
        ("=ROWS(ISNUMBER({1,2;3,4;5,6}))", "3"),
        ("=COLUMNS(ISNUMBER({1,2;3,4;5,6}))", "2"),
    ]);
}

#[test]
fn isnumber_of_an_array_in_one_cell_gives_the_top_left_value_and_spills() {
    let mut model = model();
    model._set("Z1", "=ISNUMBER({1,\"a\",3})");
    model._set("Z3", "=ISNUMBER(G1:G2)");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "TRUE");
    assert_eq!(model._get_text("AA1"), "FALSE");
    assert_eq!(model._get_text("AB1"), "TRUE");
    assert_eq!(model._get_text("Z3"), "FALSE");
    assert_eq!(model._get_text("Z4"), "TRUE");
}

#[test]
fn the_scalar_results_and_the_argument_count_error_do_not_change() {
    assert_results(&[
        ("=ISNUMBER(5)", "TRUE"),
        ("=ISNUMBER(\"5\")", "FALSE"),
        ("=ISNUMBER(TRUE)", "FALSE"),
        ("=ISNUMBER(1/0)", "FALSE"),
        ("=ISNUMBER(Y99)", "FALSE"),
        ("=ISNUMBER(L1)", "TRUE"),
        ("=ISNUMBER(G1)", "FALSE"),
        ("=ISNUMBER(--\"5\")", "TRUE"),
        ("=ISNUMBER()", "#ERROR!"),
        ("=ISNUMBER(1,2)", "#ERROR!"),
    ]);
}

#[test]
fn isnumber_of_a_range_follows_a_change_of_the_range() {
    let mut model = model();
    model._set("Z1", "=SUMPRODUCT(--ISNUMBER(L1:L4))");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "4");
    model._set("L1", "x");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "3");
}
