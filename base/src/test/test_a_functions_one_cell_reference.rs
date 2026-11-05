#![allow(clippy::unwrap_used)]

//! IronXL: `AVERAGEA`, `MAXA`, `MINA`, `STDEVA`, `STDEVPA`, `VARA`, and `VARPA` read a reference
//! argument with one rule, as in Excel. A one-cell reference, a defined name of one cell, and a
//! range count text as 0, `TRUE` as 1, and `FALSE` as 0, and skip an empty cell. A direct
//! argument keeps the direct rule: text that is a number counts, other text gives `#VALUE!`.

use crate::model::Model;
use crate::test::util::new_empty_model;

const A_FUNCTIONS: [&str; 7] = [
    "MINA", "MAXA", "AVERAGEA", "VARA", "VARPA", "STDEVA", "STDEVPA",
];

/// The results of the seven A functions, in the order of `A_FUNCTIONS`, for the values 0 and 5.
const TEXT_AND_FIVE: [&str; 7] = ["0", "5", "2.5", "12.5", "6.25", "3.535533906", "2.5"];
/// The results for the values 1 and 5.
const ONE_AND_FIVE: [&str; 7] = ["1", "5", "3", "8", "4", "2.828427125", "2"];
/// The results for the values 3 and 5.
const THREE_AND_FIVE: [&str; 7] = ["3", "5", "4", "2", "1", "1.414213562", "1"];
/// The results for the value 5 only.
const FIVE_ONLY: [&str; 7] = ["5", "5", "5", "#DIV/0!", "0", "#DIV/0!", "0"];

/// A model with `Sheet1` and `Sheet2`. `Sheet1!B1` and `Sheet2!B1` are text, `B2` is 5, `B3` is
/// TRUE, `B4` is FALSE, `B6` is empty, and `C1` is `#DIV/0!`. The name `nm` is `Sheet1!$B$1`,
/// `rng` is `Sheet1!$B$1:$B$2`, and the name `local` of `Sheet1` is `Sheet1!$B$3`.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.add_sheet("Sheet2").unwrap();
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("B3", "TRUE");
    model._set("B4", "FALSE");
    model._set("C1", "=1/0");
    model._set("Sheet2!B1", "text");
    model.new_defined_name("nm", None, "Sheet1!$B$1").unwrap();
    model
        .new_defined_name("rng", None, "Sheet1!$B$1:$B$2")
        .unwrap();
    model
        .new_defined_name("local", Some(0), "Sheet1!$B$3")
        .unwrap();
    model
}

/// The text of each formula, evaluated in one model.
fn results(formulas: &[String]) -> Vec<String> {
    let mut model = model();
    for (index, formula) in formulas.iter().enumerate() {
        model._set(&format!("E{}", index + 1), formula);
    }
    model.evaluate();
    (0..formulas.len())
        .map(|index| model._get_text(&format!("E{}", index + 1)))
        .collect()
}

/// Checks that each A function with the arguments `args` gives the matching value of `expected`.
fn assert_a_functions(args: &str, expected: &[&str; 7]) {
    let formulas: Vec<String> = A_FUNCTIONS
        .iter()
        .map(|name| format!("={name}({args})"))
        .collect();
    let actual = results(&formulas);
    for (index, formula) in formulas.iter().enumerate() {
        assert_eq!(actual[index], expected[index], "{formula}");
    }
}

#[test]
fn a_one_cell_reference_to_text_counts_as_zero() {
    assert_a_functions("B1,B2", &TEXT_AND_FIVE);
    assert_a_functions("B1:B2", &TEXT_AND_FIVE);
}

#[test]
fn a_one_cell_reference_to_a_boolean_counts_as_one_or_zero() {
    assert_a_functions("B3,B2", &ONE_AND_FIVE);
    assert_a_functions("B4,B2", &TEXT_AND_FIVE);
}

#[test]
fn a_one_cell_reference_to_an_empty_cell_is_skipped() {
    assert_a_functions("B6,B2", &FIVE_ONLY);
}

#[test]
fn absolute_and_other_sheet_one_cell_references_get_the_same_rule() {
    assert_a_functions("$B$1,B2", &TEXT_AND_FIVE);
    assert_a_functions("B$3,$B2", &ONE_AND_FIVE);
    assert_a_functions("Sheet2!$B$1,B2", &TEXT_AND_FIVE);
    assert_a_functions("Sheet2!B1,B2", &TEXT_AND_FIVE);
    assert_a_functions("Sheet2!B6,B2", &FIVE_ONLY);
}

#[test]
fn a_one_cell_reference_and_its_range_of_one_cell_give_the_same_result() {
    let mut formulas = Vec::new();
    for name in A_FUNCTIONS {
        for cell in ["B1", "B3", "B4", "B6", "C1", "Sheet2!B1"] {
            formulas.push(format!("={name}({cell},B2)"));
            let range = match cell.split_once('!') {
                Some((sheet, cell)) => format!("{sheet}!{cell}:{cell}"),
                None => format!("{cell}:{cell}"),
            };
            formulas.push(format!("={name}({range},B2)"));
        }
    }
    let actual = results(&formulas);
    for pair in 0..formulas.len() / 2 {
        assert_eq!(
            actual[2 * pair],
            actual[2 * pair + 1],
            "{} and {}",
            formulas[2 * pair],
            formulas[2 * pair + 1]
        );
    }
}

#[test]
fn a_defined_name_of_one_cell_is_a_reference() {
    assert_a_functions("nm,B2", &TEXT_AND_FIVE);
    assert_a_functions("local,B2", &ONE_AND_FIVE);
    assert_a_functions("Sheet1!local,B2", &ONE_AND_FIVE);
}

#[test]
fn a_defined_name_of_a_range_keeps_its_result() {
    assert_a_functions("rng", &TEXT_AND_FIVE);
}

#[test]
fn a_missing_defined_name_gives_name_error() {
    assert_a_functions("missing,B2", &["#NAME?"; 7]);
    assert_a_functions("Sheet2!local,B2", &["#NAME?"; 7]);
}

#[test]
fn direct_text_that_is_a_number_counts() {
    assert_a_functions("\"3\",B2", &THREE_AND_FIVE);
}

#[test]
fn a_direct_boolean_counts_as_one_or_zero() {
    assert_a_functions("TRUE,B2", &ONE_AND_FIVE);
    assert_a_functions("FALSE,B2", &TEXT_AND_FIVE);
}

#[test]
fn direct_text_that_is_not_a_number_gives_value_error() {
    assert_a_functions("\"x\",B2", &["#VALUE!"; 7]);
}

#[test]
fn an_error_in_a_one_cell_reference_or_a_direct_argument_gives_that_error() {
    assert_a_functions("C1,B2", &["#DIV/0!"; 7]);
    assert_a_functions("NA(),B2", &["#N/A"; 7]);
    assert_a_functions("B1:C1", &["#DIV/0!"; 7]);
}

#[test]
fn a_range_over_two_sheets_gives_value_error() {
    // The two ends of the range are on two sheets.
    assert_a_functions("B1:Sheet2!B2", &["#VALUE!"; 7]);
}

#[test]
fn ranges_and_whole_columns_keep_their_results() {
    assert_a_functions("B1:B2", &TEXT_AND_FIVE);
    assert_a_functions("B2:B3", &ONE_AND_FIVE);
    // B:B holds text, 5, TRUE, and FALSE, so the values are 0, 5, 1, and 0.
    assert_a_functions(
        "B:B",
        &[
            "0",
            "5",
            "1.5",
            "5.666666667",
            "4.25",
            "2.380476143",
            "2.061552813",
        ],
    );
    // Row 1 of Sheet2 holds only text.
    assert_a_functions(
        "Sheet2!1:1",
        &["0", "0", "0", "#DIV/0!", "0", "#DIV/0!", "0"],
    );
}

#[test]
fn offset_and_indirect_arguments_keep_their_results() {
    assert_a_functions("OFFSET(B1,0,0),B2", &TEXT_AND_FIVE);
    assert_a_functions("INDIRECT(\"B1\"),B2", &TEXT_AND_FIVE);
    assert_a_functions("INDIRECT(\"B3\"),B2", &ONE_AND_FIVE);
}

#[test]
fn array_arguments_keep_their_results() {
    // MINA and MAXA count text in an array as 0 and TRUE as 1. The variance functions count
    // only the numbers of an array. AVERAGEA does not read an array.
    assert_a_functions(
        "{\"a\",TRUE,5}",
        &["0", "5", "#N/IMPL!", "#DIV/0!", "0", "#DIV/0!", "0"],
    );
    assert_a_functions(
        "{1,\"a\",TRUE,5}",
        &["0", "5", "#N/IMPL!", "8", "4", "2.828427125", "2"],
    );
    assert_a_functions(
        "{1,#N/A}",
        &["#N/A", "#N/A", "#N/IMPL!", "#N/A", "#N/A", "#N/A", "#N/A"],
    );
}

#[test]
fn an_empty_argument_is_skipped() {
    assert_a_functions("B2,", &FIVE_ONLY);
}

#[test]
fn the_other_statistical_functions_do_not_change() {
    let formulas: Vec<String> = [
        "=MIN(B1,B2)",
        "=MAX(B1,B2)",
        "=AVERAGE(B1,B2)",
        "=VAR(B1,B2)",
        "=STDEV(B1,B2)",
        "=MIN(B1:B2)",
        "=VAR(B1:B3,7)",
    ]
    .iter()
    .map(|formula| formula.to_string())
    .collect();
    assert_eq!(
        results(&formulas),
        ["5", "5", "5", "#DIV/0!", "#DIV/0!", "5", "2"]
    );
}

#[test]
fn a_one_cell_reference_reads_a_spilled_value() {
    let mut model = model();
    model._set("G1", "=SEQUENCE(3)");
    model._set("E1", "=MINA(G2,9)");
    model._set("E2", "=VARA(G3,G2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "2");
    assert_eq!(model._get_text("E2"), "0.5");
}

#[test]
fn a_change_to_a_one_cell_reference_recalculates_the_a_function() {
    let mut model = model();
    model._set("E1", "=MINA(B1,B2)");
    model._set("E2", "=VARA(nm,B2)");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "0");
    model._set("B1", "-1");
    model.evaluate();
    assert_eq!(model._get_text("E1"), "-1");
    assert_eq!(model._get_text("E2"), "18");
}
