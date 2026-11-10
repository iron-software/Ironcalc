#![allow(clippy::panic)]

// IronXL: the formula text keeps each parenthesis that the operator order needs, so the text
// parses back to the same tree. Order: unary minus, `%`, `^`, `*` and `/`, `+` and `-`, `&`, and
// comparison; operators of one level read from left to right.

use std::collections::HashMap;

use crate::expressions::parser::stringify::to_rc_format;
use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::types::CellReferenceRC;

fn cell_reference() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 1,
    }
}

// Each formula gives the expected text, and the text parses back to the same tree.
fn assert_round_trip(cases: &[(&str, &str)]) {
    let cell_reference = cell_reference();
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    for (formula, expected) in cases {
        let tree = parser.parse(formula, &cell_reference);
        let text = to_english_localized_string(&tree, &cell_reference);
        assert_eq!(text, *expected, "text of {formula}");
        let tree_of_text = parser.parse(&text, &cell_reference);
        assert_eq!(tree_of_text, tree, "tree of the text of {formula}");
    }
}

#[test]
fn minus_keeps_the_parentheses_of_a_comparison_and_a_concatenation() {
    assert_round_trip(&[
        ("-(L1>1)", "-(L1>1)"),
        ("-L1>1", "-L1>1"),
        ("-($L$1>1)", "-($L$1>1)"),
        ("-$L$1>1", "-$L$1>1"),
        ("-(L1&\"x\")", "-(L1&\"x\")"),
        ("-L1&\"x\"", "-L1&\"x\""),
    ]);
}

#[test]
fn percent_keeps_the_parentheses_of_each_binary_operator() {
    assert_round_trip(&[
        ("(L1>1)%", "(L1>1)%"),
        ("L1>1%", "L1>1%"),
        ("(L1&\"5\")%", "(L1&\"5\")%"),
        ("L1&\"5\"%", "L1&\"5\"%"),
        ("(L1+1)%", "(L1+1)%"),
        ("L1+1%", "L1+1%"),
        ("(L1*2)%", "(L1*2)%"),
        ("(L1^2)%", "(L1^2)%"),
        ("(-L1)%", "-L1%"),
    ]);
}

#[test]
fn concatenation_keeps_the_parentheses_of_a_comparison() {
    assert_round_trip(&[
        ("(L1>1)&\"x\"", "(L1>1)&\"x\""),
        ("L1>(1&\"x\")", "L1>1&\"x\""),
        ("\"x\"&(L1>1)", "\"x\"&(L1>1)"),
        ("\"x\"&L1>1", "\"x\"&L1>1"),
        ("L1&(\"a\"&\"b\")", "L1&(\"a\"&\"b\")"),
        ("(L1&\"a\")&\"b\"", "L1&\"a\"&\"b\""),
    ]);
}

#[test]
fn arithmetic_keeps_the_parentheses_of_a_concatenation() {
    assert_round_trip(&[
        ("1+(L1&\"1\")", "1+(L1&\"1\")"),
        ("1+L1&\"1\"", "1+L1&\"1\""),
        ("(L1&\"1\")+1", "(L1&\"1\")+1"),
        ("1-(L1&\"1\")", "1-(L1&\"1\")"),
        ("(L1&\"1\")*2", "(L1&\"1\")*2"),
        ("L1&(\"1\"*2)", "L1&\"1\"*2"),
        ("2*(L1&\"1\")", "2*(L1&\"1\")"),
        ("2*L1&\"1\"", "2*L1&\"1\""),
        ("2/(L1&\"1\")", "2/(L1&\"1\")"),
    ]);
}

#[test]
fn comparison_keeps_the_parentheses_of_a_comparison_on_the_right() {
    assert_round_trip(&[
        ("L1=(1>0)", "L1=(1>0)"),
        ("(L1=1)>0", "L1=1>0"),
        ("L1=1>0", "L1=1>0"),
    ]);
}

#[test]
fn formulas_that_round_trip_keep_their_text() {
    assert_round_trip(&[
        ("-(L1^2)", "-(L1^2)"),
        ("(-L1)^2", "-L1^2"),
        ("L1-(1-1)", "L1-(1-1)"),
        ("L1/(2*2)", "L1/(2*2)"),
        ("L1^(2^2)", "L1^(2^2)"),
        ("-(-L1)", "-(-L1)"),
        (
            "(A1:A3=\"Finance\")+(B1:B3=\"HR\")",
            "(A1:A3=\"Finance\")+(B1:B3=\"HR\")",
        ),
    ]);
}

#[test]
fn minus_of_a_product_keeps_its_text() {
    // `-(L1*3)` and `-L1*3` give one value, so the text stays `-L1*3`.
    let cell_reference = cell_reference();
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let tree = parser.parse("-(L1*3)", &cell_reference);
    assert_eq!(to_english_localized_string(&tree, &cell_reference), "-L1*3");
}

#[test]
fn r1c1_text_keeps_the_parentheses() {
    let cell_reference = cell_reference();
    let mut parser = new_parser(vec!["Sheet1".to_string()], vec![], HashMap::new());
    let with = to_rc_format(&parser.parse("-(L1>1)", &cell_reference));
    let without = to_rc_format(&parser.parse("-L1>1", &cell_reference));
    assert_eq!(with, "-(R[0]C[11]>1)");
    assert_eq!(without, "-R[0]C[11]>1");
}
