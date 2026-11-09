#![allow(clippy::panic)]

//! IronXL: a chain of range operators reads from left to right, as in Excel. A side can be a
//! function call in any position, so `E1:OFFSET(F2,0,0):C1` is `(E1:OFFSET(F2,0,0)):C1`.

use std::collections::HashMap;

use crate::expressions::parser::tests::utils::{new_parser, to_english_localized_string};
use crate::expressions::parser::Node;
use crate::expressions::types::CellReferenceRC;

fn cell_reference() -> CellReferenceRC {
    CellReferenceRC {
        sheet: "Sheet1".to_string(),
        row: 1,
        column: 8,
    }
}

fn parse(formula: &str) -> Node {
    let worksheets = vec!["Sheet1".to_string(), "Sheet2".to_string()];
    let mut parser = new_parser(worksheets, vec![], HashMap::new());
    parser.parse(formula, &cell_reference())
}

/// The order of the range operators in a node: each `OpRangeKind` is in parentheses, and each
/// other node is its formula text.
fn shape(node: &Node) -> String {
    match node {
        Node::OpRangeKind { left, right } => format!("({}:{})", shape(left), shape(right)),
        _ => to_english_localized_string(node, &cell_reference()),
    }
}

#[test]
fn a_range_chain_with_a_function_call_reads_from_left_to_right() {
    let cases = [
        ("E1:OFFSET(F2,0,0):C1", "((E1:OFFSET(F2,0,0)):C1)"),
        ("E1:INDEX(E1:F2,2,2):C1", "((E1:INDEX(E1:F2,2,2)):C1)"),
        (
            "OFFSET(E1,0,0):OFFSET(F2,0,0):C1",
            "((OFFSET(E1,0,0):OFFSET(F2,0,0)):C1)",
        ),
        (
            "E1:OFFSET(F2,0,0):OFFSET(C1,0,0)",
            "((E1:OFFSET(F2,0,0)):OFFSET(C1,0,0))",
        ),
        ("E1:OFFSET(F2,0,0):C1:C2", "((E1:OFFSET(F2,0,0)):C1:C2)"),
        (
            "E1:OFFSET(F2,0,0):Sheet2!E1",
            "((E1:OFFSET(F2,0,0)):Sheet2!E1)",
        ),
    ];
    for (formula, expected) in cases {
        assert_eq!(shape(&parse(formula)), expected, "{formula}");
    }
}

#[test]
fn a_range_chain_reads_from_left_to_right_in_a_function_argument() {
    match parse("SUM(E1:OFFSET(F2,0,0):C1,1)") {
        Node::FunctionKind { args, .. } => {
            assert_eq!(args.len(), 2);
            assert_eq!(shape(&args[0]), "((E1:OFFSET(F2,0,0)):C1)");
            assert_eq!(shape(&args[1]), "1");
        }
        node => panic!("SUM gave {node:?}"),
    }
}

#[test]
fn a_range_chain_keeps_the_forms_that_parsed_before() {
    let cases = [
        ("E1:OFFSET(F2,0,0)", "(E1:OFFSET(F2,0,0))"),
        ("OFFSET(F2,0,0):E1:C1", "(OFFSET(F2,0,0):C1:E1)"),
        ("(E1:OFFSET(F2,0,0)):C1", "((E1:OFFSET(F2,0,0)):C1)"),
        ("E1:F2", "E1:F2"),
    ];
    for (formula, expected) in cases {
        assert_eq!(shape(&parse(formula)), expected, "{formula}");
    }
}

#[test]
fn the_text_of_a_range_chain_parses_again_to_the_same_node() {
    let node = parse("(E1:OFFSET(F2,0,0)):C1");
    let text = to_english_localized_string(&node, &cell_reference());
    assert_eq!(text, "E1:OFFSET(F2,0,0):C1");
    assert_eq!(parse(&text), node);
}

#[test]
fn a_range_chain_that_ends_in_a_range_operator_does_not_parse() {
    for formula in [
        "SUM(E1:OFFSET(F2,0,0):)",
        "E1:OFFSET(F2,0,0):",
        "OFFSET(F2,0,0):",
    ] {
        match parse(formula) {
            Node::ParseErrorKind { .. } => {}
            node => panic!("{formula} gave {node:?}"),
        }
    }
}
