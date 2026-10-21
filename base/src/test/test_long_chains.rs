#![allow(clippy::unwrap_used)]

use crate::model::Model;
use crate::test::util::new_empty_model;

// The evaluation of a long dependency chain must not recurse once per link.
// These tests run on a small stack: one recursive call per link overflows it.
const SMALL_STACK: usize = 256 * 1024;
const LENGTH: i32 = 100_000;

fn on_small_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new()
        .stack_size(SMALL_STACK)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap();
}

fn set(model: &mut Model, sheet: u32, row: i32, column: i32, value: &str) {
    model
        .set_user_input(sheet, row, column, value.to_string())
        .unwrap();
}

// A1 = 1, A2 = A1+1, ..., A{length} = A{length-1}+1
fn running_total<'a>(length: i32) -> Model<'a> {
    let mut model = new_empty_model();
    set(&mut model, 0, 1, 1, "1");
    for row in 2..=length {
        set(&mut model, 0, row, 1, &format!("=A{}+1", row - 1));
    }
    model
}

#[test]
fn running_total_last_cell_with_evaluate_cells() {
    on_small_stack(|| {
        let mut model = running_total(LENGTH);
        model.evaluate_cells(&[(0, LENGTH, 1)]).unwrap();
        assert_eq!(model._get_text_at(0, LENGTH, 1), "100000");
        assert_eq!(model._get_text_at(0, 2, 1), "2");
    });
}

#[test]
fn running_total_with_evaluate() {
    on_small_stack(|| {
        let mut model = running_total(LENGTH);
        model.evaluate();
        assert_eq!(model._get_text_at(0, LENGTH, 1), "100000");
        assert_eq!(model._get_text_at(0, 50_000, 1), "50000");
    });
}

#[test]
fn reverse_chain() {
    on_small_stack(|| {
        // A1 = A2+1, ..., A{LENGTH-1} = A{LENGTH}+1, A{LENGTH} = 1
        let build = || {
            let mut model = new_empty_model();
            for row in 1..LENGTH {
                set(&mut model, 0, row, 1, &format!("=A{}+1", row + 1));
            }
            set(&mut model, 0, LENGTH, 1, "=1");
            model
        };
        let mut model = build();
        model.evaluate();
        assert_eq!(model._get_text_at(0, 1, 1), "100000");

        let mut model = build();
        model.evaluate_cells(&[(0, 1, 1)]).unwrap();
        assert_eq!(model._get_text_at(0, 1, 1), "100000");
        assert_eq!(model._get_text_at(0, LENGTH, 1), "1");
    });
}

#[test]
fn chain_across_two_sheets_through_a_range_function() {
    on_small_stack(|| {
        // Sheet1!A1 = 1, Sheet2!An = SUM(Sheet1!An:An)+1, Sheet1!An = Sheet2!A(n-1)+1
        let rows = LENGTH / 2;
        let build = || {
            let mut model = new_empty_model();
            model.new_sheet();
            set(&mut model, 0, 1, 1, "1");
            for row in 1..=rows {
                if row > 1 {
                    set(&mut model, 0, row, 1, &format!("=Sheet2!A{}+1", row - 1));
                }
                set(
                    &mut model,
                    1,
                    row,
                    1,
                    &format!("=SUM(Sheet1!A{row}:A{row})+1"),
                );
            }
            model
        };
        let mut model = build();
        model.evaluate_cells(&[(1, rows, 1)]).unwrap();
        assert_eq!(model._get_text_at(1, rows, 1), "100000");
        assert_eq!(model._get_text_at(0, rows, 1), "99999");

        let mut model = build();
        model.evaluate();
        assert_eq!(model._get_text_at(1, rows, 1), "100000");
    });
}

#[test]
fn chain_through_whole_column_and_whole_row_ranges() {
    on_small_stack(|| {
        // B1 = 1, Bn = ROWS(C:C)*0+COLUMNS(1:1)*0+B(n-1)+1. The walk must visit only the
        // cells that exist in a whole-column or whole-row range, not every row or column.
        // 20,000 links keep the set-up short: the input of these formulas is slow.
        let rows = 20_000;
        let mut model = new_empty_model();
        set(&mut model, 0, 1, 2, "1");
        set(&mut model, 0, 5, 3, "=B1*0");
        set(&mut model, 0, 1, 7, "=B1*0");
        for row in 2..=rows {
            set(
                &mut model,
                0,
                row,
                2,
                &format!("=ROWS(C:C)*0+COLUMNS(1:1)*0+B{}+1", row - 1),
            );
        }
        model.evaluate_cells(&[(0, rows, 2)]).unwrap();
        assert_eq!(model._get_text_at(0, rows, 2), "20000");
    });
}

#[test]
fn chain_through_a_defined_name() {
    on_small_stack(|| {
        let mut model = new_empty_model();
        model
            .new_defined_name("Previous", None, "Sheet1!$A$99999")
            .unwrap();
        set(&mut model, 0, 1, 1, "1");
        for row in 2..LENGTH {
            set(&mut model, 0, row, 1, &format!("=A{}+1", row - 1));
        }
        set(&mut model, 0, LENGTH, 1, "=Previous+1");
        model.evaluate_cells(&[(0, LENGTH, 1)]).unwrap();
        assert_eq!(model._get_text_at(0, LENGTH, 1), "100000");
    });
}

#[test]
fn a_cycle_is_still_a_circular_reference_error() {
    let build = || {
        let mut model = new_empty_model();
        model._set("A1", "=B1+1");
        model._set("B1", "=A1+1");
        model._set("C1", "=A1+1");
        model._set("D1", "=IFERROR(E1, 5)");
        model._set("E1", "=D1+1");
        model
    };
    let mut model = build();
    model.evaluate();
    assert_eq!(model._get_text("A1"), "#CIRC!");
    assert_eq!(model._get_text("B1"), "#CIRC!");
    assert_eq!(model._get_text("C1"), "#CIRC!");
    // D1 is evaluated first: E1 finds the cycle and D1 catches the error of E1.
    assert_eq!(model._get_text("D1"), "5");
    assert_eq!(model._get_text("E1"), "#CIRC!");

    let mut model = build();
    model.evaluate_cells(&[(0, 1, 5), (0, 1, 3)]).unwrap();
    assert_eq!(model._get_text("C1"), "#CIRC!");
    // E1 is evaluated first: D1 finds the cycle and catches the error.
    assert_eq!(model._get_text("D1"), "5");
    assert_eq!(model._get_text("E1"), "6");
}

#[test]
fn a_cycle_at_the_end_of_a_long_chain() {
    on_small_stack(|| {
        // B1 = A100000 + C1 and C1 = B1 form a cycle on top of a long running total.
        let mut model = running_total(LENGTH);
        set(&mut model, 0, 1, 2, &format!("=A{LENGTH}+C1"));
        set(&mut model, 0, 1, 3, "=B1");
        model.evaluate_cells(&[(0, 1, 2)]).unwrap();
        assert_eq!(model._get_text_at(0, 1, 2), "#CIRC!");
        assert_eq!(model._get_text_at(0, 1, 3), "#CIRC!");
        assert_eq!(model._get_text_at(0, LENGTH, 1), "100000");
    });
}

#[test]
fn a_cycle_inside_a_chain() {
    // A1..A10000 is a running total, A10001 and B10001 form a cycle, A10002..A10100 read
    // the cycle. The cells that read the cycle are evaluated recursively, as before.
    let build = || {
        let mut model = running_total(10_000);
        model._set("A10001", "=A10000+B10001");
        model._set("B10001", "=A10001");
        for row in 10_002..=10_100 {
            model._set(&format!("A{row}"), &format!("=A{}+1", row - 1));
        }
        model
    };
    let mut model = build();
    model.evaluate_cells(&[(0, 10_100, 1)]).unwrap();
    assert_eq!(model._get_text("A10000"), "10000");
    assert_eq!(model._get_text("A10001"), "#CIRC!");
    assert_eq!(model._get_text("B10001"), "#CIRC!");
    assert_eq!(model._get_text("A10100"), "#CIRC!");

    let mut model = build();
    model.evaluate();
    assert_eq!(model._get_text("A10000"), "10000");
    assert_eq!(model._get_text("A10100"), "#CIRC!");
}

fn mixed_model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.new_sheet();
    model.new_defined_name("Rate", None, "Sheet2!$B$1").unwrap();
    model
        .new_defined_name("Block", None, "Sheet1!$A$1:$A$5")
        .unwrap();
    model
        .new_defined_name("Twice", None, "=LAMBDA(x, x * 2)")
        .unwrap();
    let inputs = [
        ("Sheet1!A1", "3"),
        ("Sheet1!A2", "=A1*2"),
        ("Sheet1!A3", "=A2+Sheet2!A1"),
        ("Sheet1!A4", "=SUM(A1:A3)"),
        ("Sheet1!A5", "=IF(A4>10, A3, A2)"),
        ("Sheet1!A6", "=SUM(Block)*Rate"),
        ("Sheet1!A7", "=INDIRECT(\"A\" & A1)+1"),
        ("Sheet1!A8", "=OFFSET(A1, 2, 0)*2"),
        ("Sheet1!A9", "=Twice(A8)"),
        ("Sheet1!A10", "=LET(v, A9, v + A6)"),
        ("Sheet1!A11", "=COUNT(A:A)"),
        ("Sheet1!A12", "=A13+1"),
        ("Sheet1!A13", "=A12+1"),
        ("Sheet1!A14", "=A13*0+A10"),
        ("Sheet1!A15", "=VLOOKUP(3, A1:B4, 1, FALSE)"),
        ("Sheet1!A16", "=CONCAT(A1, \"-\", A2)"),
        ("Sheet1!C1", "=Sheet2!C5+1"),
        ("Sheet2!A1", "=Sheet1!A1+1"),
        ("Sheet2!B1", "=0.5"),
        ("Sheet2!C1", "=Sheet1!A4"),
        ("Sheet2!C2", "=C1*2"),
        ("Sheet2!C3", "=C2*2"),
        ("Sheet2!C4", "=C3*2"),
        ("Sheet2!C5", "=C4*2"),
    ];
    for (cell, value) in inputs {
        model._set(cell, value);
    }
    model
}

#[test]
fn values_equal_whole_workbook_evaluation_for_a_mixed_workbook() {
    let mut whole = mixed_model();
    whole.evaluate();

    let mut partial = mixed_model();
    let mut formulas = Vec::new();
    for sheet in 0..2 {
        for row in 1..=16 {
            for column in 1..=3 {
                if partial
                    .get_cell_formula(sheet, row, column)
                    .unwrap()
                    .is_some()
                {
                    formulas.push((sheet, row, column));
                }
            }
        }
    }
    formulas.reverse();
    partial.evaluate_cells(&formulas).unwrap();

    for sheet in 0..2 {
        for row in 1..=16 {
            for column in 1..=3 {
                assert_eq!(
                    partial._get_text_at(sheet, row, column),
                    whole._get_text_at(sheet, row, column),
                    "({sheet}, {row}, {column})"
                );
            }
        }
    }
    assert_eq!(whole._get_text("A4"), "19");
    assert_eq!(whole._get_text("A6"), "24");
    assert_eq!(whole._get_text("A7"), "11");
    assert_eq!(whole._get_text("A8"), "20");
    assert_eq!(whole._get_text("A9"), "40");
    assert_eq!(whole._get_text("A10"), "64");
    assert_eq!(whole._get_text("A12"), "#CIRC!");
    assert_eq!(whole._get_text("A14"), "#CIRC!");
    assert_eq!(whole._get_text("Sheet1!C1"), "305");
}
