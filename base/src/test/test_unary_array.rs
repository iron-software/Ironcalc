#![allow(clippy::unwrap_used)]

//! IronXL: each minus sign negates its operand and casts it to a number, as in Excel, so `--TRUE`
//! gives 1. The unary minus and the percent operator apply to each element of an array or a range,
//! and keep its shape. Each element follows the element rule of the binary operators.

use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with `Sheet1!L1:L4` = 2, 1, 2, 1 and `G1:G4` = a, b, a, c. Column A is empty.
fn model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    for (index, (number, text)) in [("2", "a"), ("1", "b"), ("2", "a"), ("1", "c")]
        .iter()
        .enumerate()
    {
        model._set(&format!("L{}", index + 1), number);
        model._set(&format!("G{}", index + 1), text);
    }
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
fn a_double_minus_of_an_array_gives_numbers() {
    assert_results(&[
        ("=SUMPRODUCT(--({2,1,2,1}>1))", "2"),
        ("=SUMPRODUCT(--(L1:L4>1))", "2"),
        ("=SUM(--(COUNTIF(G1:G4,G1:G4)>1))", "2"),
        ("=SUMPRODUCT((L1:L4>1)*1)", "2"),
    ]);
}

#[test]
fn the_unary_operators_apply_to_each_element() {
    assert_results(&[
        ("=INDEX(-{1,2},1,2)", "-2"),
        ("=SUMPRODUCT(-{1;2})", "-3"),
        ("=INDEX(-{1,2;3,4},2,1)", "-3"),
        ("=SUM(-L1:L4)", "-6"),
        ("=SUM(-(L1:L4))", "-6"),
        ("=SUM({50,20}%)", "0.7"),
        ("=INDEX(L1:L4%,2,1)", "0.01"),
        ("=SUM(-{1,\"2\"})", "-3"),
        ("=SUM(-{1,TRUE})", "-2"),
        ("=SUM(-{1,\"a\"})", "#VALUE!"),
        ("=SUM(-{1,#N/A})", "#N/A"),
        ("=SUM({1,\"a\"}%)", "#VALUE!"),
        ("=SUM(-A1:A2)", "0"),
        ("=SUM(-Sheet1!L1:L4)", "-6"),
    ]);
}

#[test]
fn an_array_result_spills_from_the_formula_cell() {
    let mut model = model();
    model._set("Z1", "=-{1,2}");
    model._set("Z3", "=-L1:L2");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "-1");
    assert_eq!(model._get_text("AA1"), "-2");
    assert_eq!(model._get_text("Z3"), "-2");
    assert_eq!(model._get_text("Z4"), "-1");
}

#[test]
fn each_minus_sign_negates_and_casts_to_a_number() {
    assert_results(&[
        ("=--TRUE", "1"),
        ("=---TRUE", "-1"),
        ("=-+-TRUE", "1"),
        ("=ISNUMBER(--\"5\")", "TRUE"),
        ("=--\"5\"", "5"),
        ("=--\"a\"", "#VALUE!"),
        ("=+TRUE", "TRUE"),
        ("=--50%", "0.5"),
    ]);
}

#[test]
fn a_double_minus_keeps_each_sign_in_the_parse_tree() {
    let mut model = model();
    model._set("Z1", "=--TRUE");
    model._set("Z2", "=-+-L1");
    assert_eq!(model._get_formula("Z1"), "=-(-TRUE)");
    assert_eq!(model._get_formula("Z2"), "=-(-L1)");
}

#[test]
fn the_scalar_results_and_the_other_operators_keep_their_results() {
    assert_results(&[
        ("=-L1", "-2"),
        ("=-\"a\"", "#VALUE!"),
        ("=-(1/0)", "#DIV/0!"),
        ("=50%", "0.5"),
        ("=-A1", "0"),
        ("=SUM({-1,2})", "1"),
        ("=SUM(--(L1:L4))", "6"),
        ("=SUM(+{1,2})", "3"),
        ("=-2^2", "4"),
        ("=-(OFFSET(A1,-1,0))", "#REF!"),
    ]);
}

#[test]
fn the_result_follows_a_change_of_the_range() {
    let mut model = model();
    model._set("Z1", "=SUMPRODUCT(--(L1:L4>1))");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "2");
    model._set("L1", "0");
    model.evaluate();
    assert_eq!(model._get_text("Z1"), "1");
}
