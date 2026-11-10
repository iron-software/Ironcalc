#![allow(clippy::unwrap_used)]
use crate::test::util::new_empty_model;

// ── LEN gives one length for each item of an array ────────────────────────────

#[test]
fn len_of_an_array_gives_one_length_for_each_item() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(LEN({\"a\",\"bb\"}))");
    model._set("A2", "=SUMPRODUCT(LEN({\"a\",\"bb\"}))");
    model._set("A3", "=INDEX(LEN({\"a\",\"bb\"}),1,2)");
    model._set("A4", "=INDEX(LEN({\"a\";\"bbb\"}),2,1)");
    model._set("A5", "=ROWS(LEN({\"a\";\"bb\";\"ccc\"}))");
    model._set("A6", "=COLUMNS(LEN({\"a\",\"bb\"}))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(model._get_text("A5"), "3");
    assert_eq!(model._get_text("A6"), "2");
}

#[test]
fn len_of_an_array_spills_the_lengths() {
    let mut model = new_empty_model();
    model._set("A1", "=LEN({\"a\",\"bb\",\"ccc\"})");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("B1"), "2");
    assert_eq!(model._get_text("C1"), "3");
}

// ── LEN gives one length for each cell of a range ─────────────────────────────

#[test]
fn len_of_a_range_gives_one_length_for_each_cell() {
    let mut model = new_empty_model();
    model._set("Z1", "abc");
    model._set("Z2", "12");
    model._set("Z3", "x");
    model._set("A1", "=SUM(LEN(Z1:Z3))");
    model._set("A2", "=SUM(LEN(Z1:Z4))");
    model._set("A3", "=INDEX(LEN(Z1:Z4),4)");
    model._set("A4", "=INDEX(LEN(Z1:Z3),2)");
    model._set("C1", "=LEN(Z1:Z3)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
    // Z4 is empty: its length is 0.
    assert_eq!(model._get_text("A2"), "6");
    assert_eq!(model._get_text("A3"), "0");
    assert_eq!(model._get_text("A4"), "2");
    // The formula cell gives the top-left value and spills the others.
    assert_eq!(model._get_text("C1"), "3");
    assert_eq!(model._get_text("C2"), "2");
    assert_eq!(model._get_text("C3"), "1");
}

// ── Each item uses the scalar text rule ───────────────────────────────────────

#[test]
fn len_items_use_the_scalar_text_rule() {
    let mut model = new_empty_model();
    model._set("A1", "=SUM(LEN({TRUE,12}))");
    model._set("A2", "=INDEX(LEN({FALSE,1.5}),1,1)");
    model._set("A3", "=INDEX(LEN({FALSE,1.5}),1,2)");
    model._set("A4", "=SUM(LEN(1/{1,4}))");
    model._set("A5", "=INDEX(LEN({\"héllo\"}),1,1)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "6");
    assert_eq!(model._get_text("A2"), "5");
    assert_eq!(model._get_text("A3"), "3");
    // 1 and 0.25
    assert_eq!(model._get_text("A4"), "5");
    assert_eq!(model._get_text("A5"), "5");
}

#[test]
fn len_keeps_an_error_item_in_its_position() {
    let mut model = new_empty_model();
    model._set("Z1", "abc");
    model._set("Z2", "=1/0");
    model._set("A1", "=INDEX(LEN({\"a\",#N/A}),1,1)");
    model._set("A2", "=INDEX(LEN({\"a\",#N/A}),1,2)");
    model._set("A3", "=SUM(LEN({\"a\",#N/A}))");
    model._set("A4", "=INDEX(LEN(Z1:Z2),1)");
    model._set("A5", "=INDEX(LEN(Z1:Z2),2)");
    model._set("A6", "=SUM(LEN(Z1:Z2))");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "1");
    assert_eq!(model._get_text("A2"), "#N/A");
    assert_eq!(model._get_text("A3"), "#N/A");
    assert_eq!(model._get_text("A4"), "3");
    assert_eq!(model._get_text("A5"), "#DIV/0!");
    assert_eq!(model._get_text("A6"), "#DIV/0!");
}

// ── Scalar behaviour is preserved ─────────────────────────────────────────────

#[test]
fn len_of_a_scalar_keeps_its_result() {
    let mut model = new_empty_model();
    model._set("Z1", "abc");
    model._set("A1", "=LEN(\"abc\")");
    model._set("A2", "=LEN(Z1)");
    model._set("A3", "=LEN(12)");
    model._set("A4", "=LEN(TRUE)");
    model._set("A5", "=LEN(Z9)");
    model._set("A6", "=LEN(1/0)");
    model._set("A7", "=LEN()");
    model._set("A8", "=LEN(1,2)");
    model._set("A9", "=INDEX(UPPER({\"a\",\"bb\"}),1,2)");
    model.evaluate();
    assert_eq!(model._get_text("A1"), "3");
    assert_eq!(model._get_text("A2"), "3");
    assert_eq!(model._get_text("A3"), "2");
    assert_eq!(model._get_text("A4"), "4");
    assert_eq!(model._get_text("A5"), "0");
    assert_eq!(model._get_text("A6"), "#DIV/0!");
    assert_eq!(model._get_text("A7"), "#ERROR!");
    assert_eq!(model._get_text("A8"), "#ERROR!");
    assert_eq!(model._get_text("A9"), "BB");
    // A scalar LEN does not spill.
    assert_eq!(model._get_text("B1"), "");
}
