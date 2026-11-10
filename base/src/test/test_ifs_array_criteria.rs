#![allow(clippy::unwrap_used)]

//! IronXL: COUNTIFS, SUMIFS, AVERAGEIF, AVERAGEIFS, MINIFS, and MAXIFS with an array or a range of
//! criteria give one result for each criterion, as COUNTIF and SUMIF do and as in Excel. Arrays in
//! more than one criteria pair broadcast: the result has the largest rows and columns, a length-1
//! axis repeats, and a position that one array does not have gives `#N/A`.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!B1` text, `B2` 5, `C1` 7, `C2` 8, `E1` 5, and `E2` text. The other cells
/// are empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model._set("B1", "text");
    model._set("B2", "5");
    model._set("C1", "7");
    model._set("C2", "8");
    model._set("E1", "5");
    model._set("E2", "text");
    model
}

/// Checks that each formula gives its expected text, evaluated in one model. Each formula gives
/// one value, so no result spills into the next formula.
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
fn the_criteria_functions_give_one_result_for_each_criterion_of_an_array() {
    assert_results(&[
        ("=SUM(COUNTIFS(B1:B2,{5,6}))", "1"),
        ("=SUM(COUNTIFS(B1:B2,{5,6},B1:B2,\"<>x\"))", "1"),
        ("=SUM(SUMIFS(B1:B2,B1:B2,{5,6}))", "5"),
        ("=INDEX(AVERAGEIF(B1:B2,{5,6}),1,1)", "5"),
        ("=SUMPRODUCT(COUNTIFS(B1:B2,{5,6}))", "1"),
        ("=SUM(COUNTIFS(B1:B2,{5,5}))", "2"),
    ]);
}

#[test]
fn each_function_gives_its_result_for_no_match() {
    assert_results(&[
        ("=INDEX(AVERAGEIF(B1:B2,{5,6}),1,2)", "#DIV/0!"),
        ("=INDEX(AVERAGEIF(B1:B2,{5,6},C1:C2),1,1)", "8"),
        ("=INDEX(AVERAGEIF(B1:B2,{5,6},C1:C2),1,2)", "#DIV/0!"),
        ("=INDEX(AVERAGEIFS(B1:B2,B1:B2,{5,6}),1,1)", "5"),
        ("=INDEX(AVERAGEIFS(C1:C2,B1:B2,{5;\"text\"}),2,1)", "7"),
        ("=SUM(MINIFS(B1:B2,B1:B2,{5,6}))", "5"),
        ("=INDEX(MINIFS(B1:B2,B1:B2,{5,6}),1,2)", "0"),
        ("=INDEX(MINIFS(C1:C2,B1:B2,{5,\"*\"}),1,2)", "7"),
        ("=SUM(MAXIFS(B1:B2,B1:B2,{5,6}))", "5"),
        ("=INDEX(MAXIFS(C1:C2,C1:C2,{\">0\",\"<0\"}),1,1)", "8"),
        ("=INDEX(MAXIFS(C1:C2,C1:C2,{\">0\",\"<0\"}),1,2)", "0"),
        ("=INDEX(SUMIFS(C1:C2,B1:B2,{5,6}),1,2)", "0"),
    ]);
}

#[test]
fn a_range_of_criteria_gives_one_result_for_each_cell() {
    assert_results(&[
        ("=SUM(COUNTIFS(B1:B2,E1:E2))", "2"),
        ("=SUM(SUMIFS(B1:B2,B1:B2,E1:E2))", "5"),
        ("=INDEX(SUMIFS(C1:C2,B1:B2,E1:E2),2,1)", "7"),
        ("=INDEX(AVERAGEIF(B1:B2,E1:E2,C1:C2),1,1)", "8"),
        ("=SUM(MAXIFS(C1:C2,B1:B2,E1:E2))", "15"),
    ]);
}

#[test]
fn arrays_in_more_than_one_criteria_pair_broadcast() {
    assert_results(&[
        ("=SUM(COUNTIFS(B1:B2,{5,6},C1:C2,{7;8}))", "1"),
        ("=INDEX(COUNTIFS(B1:B2,{5,6},C1:C2,{7;8}),2,1)", "1"),
        ("=INDEX(COUNTIFS(B1:B2,{5,6},C1:C2,{7;8}),1,1)", "0"),
        ("=INDEX(COUNTIFS(B1:B2,{5,6},B1:B2,{5,6,7}),1,1)", "1"),
        ("=INDEX(COUNTIFS(B1:B2,{5,6},B1:B2,{5,6,7}),1,3)", "#N/A"),
        ("=INDEX(SUMIFS(C1:C2,B1:B2,{5,6},C1:C2,{8;7}),1,1)", "8"),
        (
            "=INDEX(SUMIFS(C1:C2,B1:B2,{5;6},C1:C2,{8;7;6}),3,1)",
            "#N/A",
        ),
        // A scalar criterion repeats at each position.
        ("=SUM(COUNTIFS(B1:B2,{5,6},C1:C2,8))", "1"),
        ("=SUM(MINIFS(C1:C2,C1:C2,\">0\",B1:B2,{5,\"text\"}))", "15"),
    ]);
}

#[test]
fn each_criterion_follows_the_criteria_grammar() {
    assert_results(&[
        ("=SUM(COUNTIFS(B1:B2,{5,#N/A}))", "1"),
        ("=SUM(COUNTIFS(B1:B2,{\">0\",\"*\"}))", "2"),
        ("=SUM(COUNTIFS(B1:B2,{TRUE,5}))", "1"),
        // A whole column counts the empty cells outside the used range for each criterion.
        ("=SUM(COUNTIFS(A:A,{\"\",5}))", "1048576"),
        ("=SUM(COUNTIFS(A:A,{\"\",5},A:A,\"\"))", "1048576"),
    ]);
}

#[test]
fn an_array_result_spills_from_the_formula_cell() {
    let mut model = model();
    model._set("H1", "=COUNTIFS(B1:B2,{5,6})");
    model._set("H3", "=AVERAGEIF(B1:B2,E1:E2,C1:C2)");
    model._set("H6", "=COUNTIFS(B1:B2,{5,6},C1:C2,{7;8})");
    model._set("H9", "=MINIFS(C1:C2,B1:B2,{5,6})");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "1");
    assert_eq!(model._get_text("I1"), "0");
    assert_eq!(model._get_text("H3"), "8");
    assert_eq!(model._get_text("H4"), "7");
    assert_eq!(model._get_text("H6"), "0");
    assert_eq!(model._get_text("I6"), "0");
    assert_eq!(model._get_text("H7"), "1");
    assert_eq!(model._get_text("I7"), "0");
    assert_eq!(model._get_text("H9"), "8");
    assert_eq!(model._get_text("I9"), "0");
}

#[test]
fn an_error_cell_of_the_sum_range_gives_an_error_element() {
    let mut model = model();
    model._set("D1", "=1/0");
    model._set("D2", "3");
    model._set("H1", "=INDEX(SUMIFS(D1:D2,B1:B2,{5,\"text\"}),1,1)");
    model._set("H2", "=INDEX(SUMIFS(D1:D2,B1:B2,{5,\"text\"}),1,2)");
    model._set("H3", "=INDEX(MAXIFS(D1:D2,B1:B2,{5,\"text\"}),1,2)");
    model.evaluate();
    assert_eq!(model._get_text("H1"), "3");
    assert_eq!(model._get_text("H2"), "#DIV/0!");
    assert_eq!(model._get_text("H3"), "#DIV/0!");
}

#[test]
fn a_range_error_and_the_size_error_are_the_whole_result() {
    assert_results(&[
        ("=SUM(COUNTIFS(B1:B2,{5,6},B1:B3,5))", "#VALUE!"),
        ("=SUM(SUMIFS(B1:B2,B1:B3,{5,6}))", "#VALUE!"),
        ("=SUM(MINIFS(B1:B2,B1:B2,{5,6},C1:C3,7))", "#VALUE!"),
        ("=SUM(COUNTIFS(OFFSET(A1,-1,0),{5,6}))", "#REF!"),
        ("=SUM(SUMIFS(OFFSET(A1,-1,0),B1:B2,{5,6}))", "#REF!"),
        ("=SUM(AVERAGEIFS(B1:B2,OFFSET(A1,-1,0),{5,6}))", "#REF!"),
        ("=SUM(COUNTIFS({5,6},{5,6}))", "#VALUE!"),
        ("=SUM(SUMIFS({5,6},B1:B2,{5,6}))", "#VALUE!"),
        ("=COUNTIFS(B1:B2,{5,6},B1:B2)", "#ERROR!"),
        ("=SUMIFS(B1:B2,B1:B2,{5,6},B1:B2)", "#ERROR!"),
    ]);
}

#[test]
fn scalar_criteria_and_countif_and_sumif_keep_their_results() {
    assert_results(&[
        ("=COUNTIFS(B1:B2,5)", "1"),
        ("=COUNTIFS(B1:B2,E1)", "1"),
        ("=SUMIFS(B1:B2,B1:B2,5)", "5"),
        ("=AVERAGEIF(B1:B2,5)", "5"),
        ("=AVERAGEIF(B1:B2,6)", "#DIV/0!"),
        ("=AVERAGEIFS(C1:C2,B1:B2,\"text\")", "7"),
        ("=MINIFS(B1:B2,B1:B2,5)", "5"),
        ("=MAXIFS(C1:C2,B1:B2,6)", "0"),
        ("=SUM(COUNTIF(B1:B2,{5,6}))", "1"),
        ("=SUM(SUMIF(B1:B2,{5,6}))", "5"),
        ("=INDEX(SUMIF(B1:B2,{5,6},C1:C2),1,1)", "8"),
        ("=SUMIF(B1:B2,{5,6},C1:C3)", "8"),
    ]);
}
