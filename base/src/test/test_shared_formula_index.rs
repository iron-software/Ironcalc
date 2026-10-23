#![allow(clippy::unwrap_used)]

// IronXL: a set formula finds an equal shared formula text through a hash index.

use std::time::{Duration, Instant};

use crate::model::Model;
use crate::test::util::new_empty_model;

fn formula_count(model: &Model, sheet: u32) -> usize {
    model
        .workbook
        .worksheet(sheet)
        .unwrap()
        .shared_formulas
        .len()
}

#[test]
fn equal_texts_share_one_index_and_new_texts_append() {
    let mut model = new_empty_model();
    // The same relative reference gives the same R1C1 text (RC[-1]) in each row.
    for row in 1..=5 {
        model.set_user_input(0, row, 2, format!("=A{row}")).unwrap();
    }
    assert_eq!(formula_count(&model, 0), 1);
    model.set_user_input(0, 1, 3, "=$A$1".to_string()).unwrap();
    model.set_user_input(0, 2, 3, "=$A$1".to_string()).unwrap();
    assert_eq!(formula_count(&model, 0), 2);
    // An array formula uses the same index.
    model
        .set_user_array_formula(0, 3, 3, 1, 1, "=$A$1")
        .unwrap();
    assert_eq!(formula_count(&model, 0), 2);
    model._set("A1", "7");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "7");
    assert_eq!(model._get_text("C2"), "7");
    assert_eq!(model._get_formula("B5"), "=A5");
}

#[test]
fn each_sheet_has_its_own_index() {
    let mut model = new_empty_model();
    model.new_sheet();
    model.set_user_input(0, 1, 1, "=1+1".to_string()).unwrap();
    model.set_user_input(1, 1, 1, "=1+1".to_string()).unwrap();
    model.set_user_input(1, 2, 1, "=2+2".to_string()).unwrap();
    assert_eq!(formula_count(&model, 0), 1);
    assert_eq!(formula_count(&model, 1), 2);
}

#[test]
fn the_index_follows_a_rename_an_insert_and_a_delete_of_a_sheet() {
    let mut model = new_empty_model();
    model
        .set_user_input(0, 1, 1, "=Sheet1!B1".to_string())
        .unwrap();
    model.rename_sheet_by_index(0, "Data").unwrap();
    // The stored text changed to the new name; an equal formula reuses it.
    model
        .set_user_input(0, 2, 1, "=Data!B2".to_string())
        .unwrap();
    assert_eq!(formula_count(&model, 0), 1);

    model.insert_sheet("First", 0, None).unwrap();
    model
        .set_user_input(1, 3, 1, "=Data!B3".to_string())
        .unwrap();
    assert_eq!(formula_count(&model, 1), 1);
    model.delete_sheet(0).unwrap();
    model
        .set_user_input(0, 4, 1, "=Data!B4".to_string())
        .unwrap();
    assert_eq!(formula_count(&model, 0), 1);
}

#[test]
fn a_loaded_model_indexes_its_formulas() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 2, "=A1*2".to_string()).unwrap();
    model.set_user_input(0, 1, 3, "=A1*3".to_string()).unwrap();
    let bytes = model.to_bytes();
    let mut model = Model::from_bytes(&bytes, "en").unwrap();
    model.set_user_input(0, 2, 3, "=A2*3".to_string()).unwrap();
    model.set_user_input(0, 2, 4, "=A2*4".to_string()).unwrap();
    assert_eq!(formula_count(&model, 0), 3);
    model._set("A2", "5");
    model.evaluate();
    assert_eq!(model._get_text("C2"), "15");
    assert_eq!(model._get_text("D2"), "20");
}

#[test]
fn a_duplicate_sheet_indexes_its_copied_formulas() {
    let mut model = new_empty_model();
    model.set_user_input(0, 1, 2, "=A1+1".to_string()).unwrap();
    let (_, copy) = model.duplicate_sheet(0).unwrap();
    model
        .set_user_input(copy, 2, 2, "=A2+1".to_string())
        .unwrap();
    assert_eq!(formula_count(&model, copy), 1);
}

#[test]
fn a_whole_column_of_distinct_texts_sets_in_bounded_time() {
    // Each row gets a different R1C1 text (R[-n]C[-1]). A linear search makes
    // this count quadratic and takes hours; the index keeps it linear.
    let mut model = new_empty_model();
    let start = Instant::now();
    for row in 1..=1_048_576 {
        model.set_user_input(0, row, 2, "=A1".to_string()).unwrap();
    }
    assert_eq!(formula_count(&model, 0), 1_048_576);
    assert!(
        start.elapsed() < Duration::from_secs(120),
        "{:?}",
        start.elapsed()
    );
    assert_eq!(model._get_formula("B1048576"), "=A1");
}
