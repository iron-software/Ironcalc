#![allow(clippy::unwrap_used)]

//! IronXL: a defined name of one cell is a reference, as a defined name of a range is, as in
//! Excel. `ISREF`, `OFFSET`, `ROW`, `ROWS`, `COLUMN`, `COLUMNS`, `CELL`, the aggregates, the
//! range operator, the lookup ranges, the first argument of `INDEX`, and the criteria ranges
//! read it as a range of one cell. A use that needs a value reads the value of the cell.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1` and `Sheet2`. `Sheet1` holds `E1` 1, `F1` 2, `E2` 3, `F2` 4, `C1` 7,
/// `C2` 8, `B1` text, `B2` 5, and `B3` TRUE. The workbook names are `Corner` (`$F$2`), `Block`
/// (`$E$1:$F$2`), `One` (`$B$2`), `Two` (`$B$1:$B$2`), `Txt` (`$B$1`), `Flag` (`$B$3`),
/// `RelRow` (`$F2`), and `Rate` (0.2). The name `Local` of `Sheet1` is `$F$2`.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    for (reference, input) in [
        ("E1", "1"),
        ("F1", "2"),
        ("E2", "3"),
        ("F2", "4"),
        ("C1", "7"),
        ("C2", "8"),
        ("B1", "text"),
        ("B2", "5"),
        ("B3", "TRUE"),
    ] {
        model._set(reference, input);
    }
    for result in model.add_defined_names(&[
        ("Corner", None, "Sheet1!$F$2"),
        ("Block", None, "Sheet1!$E$1:$F$2"),
        ("One", None, "Sheet1!$B$2"),
        ("Two", None, "Sheet1!$B$1:$B$2"),
        ("Txt", None, "Sheet1!$B$1"),
        ("Flag", None, "Sheet1!$B$3"),
        ("RelRow", None, "Sheet1!$F2"),
        ("Rate", None, "0.2"),
        ("Local", Some(0), "Sheet1!$F$2"),
    ]) {
        result.unwrap();
    }
    model
}

/// Checks that each formula in row 1 of column `H` and below gives its expected text, evaluated
/// in one model. A formula in row `n` is in `H{n}`.
fn assert_results(cases: &[(&str, &str)]) {
    let mut model = model();
    for (index, (formula, _)) in cases.iter().enumerate() {
        model._set(&format!("H{}", index + 1), formula);
    }
    model.evaluate();
    for (index, (formula, expected)) in cases.iter().enumerate() {
        assert_eq!(
            model._get_text(&format!("H{}", index + 1)),
            *expected,
            "{formula}"
        );
    }
}

#[test]
fn a_name_of_one_cell_is_a_reference_for_isref_offset_rows_and_the_range_operator() {
    assert_results(&[
        ("=ISREF(Corner)", "TRUE"),
        ("=SUM(OFFSET(Corner,0,0))", "4"),
        ("=SUM(OFFSET(Corner,-1,-1))", "1"),
        ("=SUM(Corner:Corner)", "4"),
        ("=SUM(Corner:E1)", "10"),
        ("=ROWS(Corner:E1)", "2"),
        ("=SUM(Corner)", "4"),
        ("=SUM(Block:C1)", "25"),
    ]);
}

#[test]
fn a_lookup_criteria_or_index_range_reads_a_name_of_one_cell_as_a_range() {
    assert_results(&[
        ("=COUNTIF(One,5)", "1"),
        ("=SUMIF(One,5)", "5"),
        ("=SUMIF(Txt,\"text\",One)", "5"),
        ("=COUNTIFS(One,5)", "1"),
        ("=VLOOKUP(5,One,1,FALSE)", "5"),
        ("=HLOOKUP(5,One,1,FALSE)", "5"),
        ("=MATCH(5,One,0)", "1"),
        ("=XLOOKUP(5,One,One)", "5"),
        ("=INDEX(One,1)", "5"),
        ("=INDEX(One,1,1)", "5"),
        ("=COUNTIF(Two,5)", "1"),
    ]);
}

#[test]
fn every_reference_argument_reads_a_name_of_one_cell_as_a_reference() {
    assert_results(&[
        ("=ROW(Corner)", "2"),
        ("=COLUMN(Corner)", "6"),
        ("=ROWS(Corner)", "1"),
        ("=COLUMNS(Corner)", "1"),
        ("=CELL(\"address\",Corner)", "$F$2"),
        ("=ISFORMULA(Corner)", "FALSE"),
        ("=SUM(Txt)", "0"),
        ("=SUM(Corner,Txt)", "4"),
        ("=SUM(Flag,Corner)", "4"),
        ("=MAX(Txt,Corner)", "4"),
        ("=AND(Flag)", "TRUE"),
        ("=ISREF(Block)", "TRUE"),
        ("=ISREF(Local)", "TRUE"),
        ("=ISREF(Sheet1!Local)", "TRUE"),
        ("=SUM(Sheet1!Local:E1)", "10"),
        ("=SUM(Local:E1)", "10"),
    ]);
}

#[test]
fn a_name_of_one_cell_with_a_relative_part_is_a_reference_at_the_calling_cell() {
    // `RelRow` is `$F2` read at `A1`, so in row 1 it refers to `F2`, and in row 2 to `F3`.
    let mut model = model();
    model._set("H1", "=SUM(RelRow:E1)");
    model._set("I1", "=ROW(RelRow)");
    model._set("J1", "=ISREF(RelRow)");
    model._set("K1", "=SUM(RelRow)");
    model._set("I2", "=ROW(RelRow)");
    model._set("K2", "=SUM(RelRow)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "10");
    assert_eq!(model._get_text("I1"), "2");
    assert_eq!(model._get_text("J1"), "TRUE");
    assert_eq!(model._get_text("K1"), "4");
    assert_eq!(model._get_text("I2"), "3");
    assert_eq!(model._get_text("K2"), "0");
}

#[test]
fn a_use_that_needs_a_value_reads_the_value_of_the_cell_of_the_name() {
    assert_results(&[
        ("=Corner", "4"),
        ("=Corner+1", "5"),
        ("=LEN(Txt)", "4"),
        ("=ISTEXT(Txt)", "TRUE"),
        ("=Txt", "text"),
        ("=COUNTA(Txt)", "1"),
        ("=COUNT(Txt)", "0"),
        ("=Rate", "0.2"),
    ]);
}

#[test]
fn a_name_that_gives_no_reference_is_not_a_reference() {
    assert_results(&[
        ("=ISREF(Rate)", "FALSE"),
        ("=ISREF(Nope)", "FALSE"),
        ("=ISREF(Sheet2!Local)", "FALSE"),
        ("=ROWS(Rate)", "1"),
        ("=COLUMNS(Rate)", "1"),
        ("=ROWS(Nope)", "#NAME?"),
        ("=SUM(Rate:E1)", "#VALUE!"),
        ("=COUNTIF(Rate,5)", "#VALUE!"),
    ]);
}

#[test]
fn a_result_that_reads_a_name_of_one_cell_follows_a_change_of_the_cell() {
    let mut model = model();
    model._set("H1", "=SUM(Corner:E1)");
    model._set("H2", "=COUNTIF(One,5)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "10");
    assert_eq!(model._get_text("H2"), "1");
    model._set("F2", "14");
    model._set("B2", "6");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "20");
    assert_eq!(model._get_text("H2"), "0");
}
