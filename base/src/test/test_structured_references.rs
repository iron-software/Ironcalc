#![allow(clippy::unwrap_used)]

//! IronXL: the engine resolves Excel's structured references from the tables that `Model::add_tables` gives:
//! columns, column ranges, special items, `@` for this row, references without a table name, the `'` escape, and
//! table and column names without case. A reference that the tables cannot resolve gives `#REF!`, and `#This Row`
//! outside the body rows gives `#VALUE!`.

use crate::model::Model;
use crate::test::util::new_empty_model;
use crate::types::{Table, TableColumn, TableStyleInfo};

/// A table with the `columns`, in `reference` of `sheet`, with one header row and `totals` totals rows.
fn table(name: &str, sheet: &str, reference: &str, columns: &[&str], totals: u32) -> Table {
    Table {
        name: name.to_string(),
        display_name: name.to_string(),
        sheet_name: sheet.to_string(),
        reference: reference.to_string(),
        totals_row_count: totals,
        header_row_count: 1,
        header_row_dxf_id: None,
        data_dxf_id: None,
        totals_row_dxf_id: None,
        columns: columns
            .iter()
            .enumerate()
            .map(|(id, name)| TableColumn {
                id: id as u32 + 1,
                name: name.to_string(),
                ..Default::default()
            })
            .collect(),
        style_info: TableStyleInfo::default(),
        has_filters: false,
    }
}

/// Sales on Sheet1!A1:D5: a header row, three body rows, and a totals row. Plain on Sheet2!A1:B3, without a totals
/// row; its second column name has the characters `'`, `[`, and `]`.
fn sales_model<'a>() -> Model<'a> {
    let mut model = new_empty_model();
    model.new_sheet();
    model.add_tables(vec![
        table(
            "Sales",
            "Sheet1",
            "A1:D5",
            &["Product", "Quantity", "Unit Price", "Amount"],
            1,
        ),
        table("Plain", "Sheet2", "A1:B3", &["Jan", "a'b[c]"], 0),
    ]);
    let inputs = [
        ("Sheet1!A1", "Product"),
        ("Sheet1!B1", "Quantity"),
        ("Sheet1!C1", "Unit Price"),
        ("Sheet1!D1", "Amount"),
        ("Sheet1!A2", "x"),
        ("Sheet1!B2", "1"),
        ("Sheet1!C2", "10"),
        ("Sheet1!D2", "=[@Quantity]*[@[Unit Price]]"),
        ("Sheet1!A3", "y"),
        ("Sheet1!B3", "2"),
        ("Sheet1!C3", "20"),
        ("Sheet1!D3", "=Sales[@Quantity]*Sales[@[Unit Price]]"),
        ("Sheet1!A4", "z"),
        ("Sheet1!B4", "3"),
        ("Sheet1!C4", "30"),
        (
            "Sheet1!D4",
            "=Sales[[#This Row],[Quantity]]*Sales[[#This Row],[Unit Price]]",
        ),
        ("Sheet1!B5", "=SUBTOTAL(109,[Quantity])"),
        ("Sheet1!D5", "=SUBTOTAL(109,Sales[Amount])"),
        ("Sheet2!A1", "Jan"),
        ("Sheet2!B1", "a'b[c]"),
        ("Sheet2!A2", "1"),
        ("Sheet2!A3", "2"),
        ("Sheet2!B2", "5"),
        ("Sheet2!B3", "7"),
    ];
    for (reference, input) in inputs {
        model._set(reference, input);
    }
    model
}

/// The formatted value of `formula` in the cell `reference` of a fresh Sales model.
fn value_at(reference: &str, formula: &str) -> String {
    let mut model = sales_model();
    model._set(reference, formula);
    model.evaluate();
    model._get_text(reference)
}

#[test]
fn this_row_and_references_without_a_table_name_evaluate_in_the_body() {
    let mut model = sales_model();
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!D2"), "10");
    assert_eq!(model._get_text("Sheet1!D3"), "40");
    assert_eq!(model._get_text("Sheet1!D4"), "90");
    // A reference without a table name in the totals row names the body of its column
    assert_eq!(model._get_text("Sheet1!B5"), "6");
    assert_eq!(model._get_text("Sheet1!D5"), "140");
}

#[test]
fn columns_and_column_ranges_give_the_body_rows() {
    assert_eq!(value_at("Sheet1!F1", "=SUM(Sales[Amount])"), "140");
    assert_eq!(value_at("Sheet1!F1", "=SUM(Sales[[Quantity]])"), "6");
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Sales[[Quantity]:[Unit Price]])"),
        "66"
    );
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Sales[[Unit Price]:[Quantity]])"),
        "66"
    );
    assert_eq!(value_at("Sheet1!F1", "=COUNTA(Sales[])"), "12");
}

#[test]
fn special_items_give_their_rows() {
    assert_eq!(value_at("Sheet1!F1", "=COUNTA(Sales[#All])"), "18");
    assert_eq!(value_at("Sheet1!F1", "=COUNTA(Sales[#Data])"), "12");
    assert_eq!(value_at("Sheet1!F1", "=COUNTA(Sales[#Headers])"), "4");
    assert_eq!(value_at("Sheet1!F1", "=SUM(Sales[#Totals])"), "146");
    assert_eq!(
        value_at("Sheet1!F1", "=COUNTA(Sales[[#Headers],[#Data]])"),
        "16"
    );
    assert_eq!(
        value_at("Sheet1!F1", "=COUNTA(Sales[[#Data],[#Totals]])"),
        "14"
    );
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Sales[[#Data],[#Totals],[Quantity]])"),
        "12"
    );
    assert_eq!(
        value_at("Sheet1!F1", "=Sales[[#Headers],[Unit Price]]"),
        "Unit Price"
    );
    assert_eq!(value_at("Sheet1!F1", "=Sales[[#Totals],[Amount]]"), "140");
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Sales[[#All],[Quantity]])"),
        "12"
    );
    // A specifier compares without case
    assert_eq!(value_at("Sheet1!F1", "=COUNTA(Sales[#data])"), "12");
}

#[test]
fn this_row_outside_the_table_takes_the_row_of_the_cell() {
    assert_eq!(value_at("Sheet1!H3", "=Sales[@Quantity]"), "2");
    assert_eq!(
        value_at("Sheet1!H4", "=SUM(Sales[@[Quantity]:[Unit Price]])"),
        "33"
    );
    assert_eq!(value_at("Sheet1!H2", "=SUM(Sales[@])"), "21");
}

#[test]
fn names_compare_without_case_and_the_escape_reads_special_characters() {
    assert_eq!(value_at("Sheet1!F1", "=SUM(sales[QUANTITY])"), "6");
    assert_eq!(value_at("Sheet1!F1", "=SUM(SALES[[unit price]])"), "60");
    // A table on another sheet
    assert_eq!(value_at("Sheet1!F1", "=SUM(Plain[a''b'[c']])"), "12");
    assert_eq!(value_at("Sheet1!F1", "=SUM(Plain[[a''b'[c']]])"), "12");
    assert_eq!(value_at("Sheet2!D1", "=SUM(Plain[Jan])"), "3");
}

#[test]
fn unresolved_references_give_ref() {
    // An unknown table or column
    assert_eq!(value_at("Sheet1!F1", "=SUM(Nope[Amount])"), "#REF!");
    assert_eq!(value_at("Sheet1!F1", "=SUM(Sales[Nope])"), "#REF!");
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Sales[[Quantity]:[Nope]])"),
        "#REF!"
    );
    // A reference without a table name outside a table
    assert_eq!(value_at("Sheet1!F2", "=[@Quantity]"), "#REF!");
    assert_eq!(value_at("Sheet1!F2", "=SUM([Quantity])"), "#REF!");
    // #Totals of a table without a totals row
    assert_eq!(value_at("Sheet1!F1", "=SUM(Plain[#Totals])"), "#REF!");
    assert_eq!(
        value_at("Sheet1!F1", "=SUM(Plain[[#Totals],[Jan]])"),
        "#REF!"
    );
}

#[test]
fn this_row_outside_the_body_rows_gives_value() {
    // Excel gives #VALUE! in the header row, the totals row, and a row below the table
    assert_eq!(value_at("Sheet1!H1", "=Sales[@Quantity]"), "#VALUE!");
    assert_eq!(value_at("Sheet1!H5", "=Sales[@Quantity]"), "#VALUE!");
    assert_eq!(value_at("Sheet1!H20", "=Sales[@Quantity]"), "#VALUE!");
    assert_eq!(
        value_at("Sheet1!H20", "=Sales[[#This Row],[Quantity]]"),
        "#VALUE!"
    );
}

#[test]
fn a_defined_name_resolves_a_table_name_but_not_a_reference_without_one() {
    let mut model = sales_model();
    let results = model.add_defined_names(&[
        ("Qty", None, "Sales[Quantity]"),
        ("Bare", None, "[Quantity]"),
    ]);
    assert_eq!(results, vec![Ok(()), Ok(())]);
    model._set("Sheet1!F1", "=SUM(Qty)");
    model._set("Sheet1!F2", "=SUM(Bare)");
    // Inside the table, a name still has no cell of its own
    model._set("Sheet1!A2", "=SUM(Bare)");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!F1"), "6");
    assert_eq!(model._get_text("Sheet1!F2"), "#REF!");
    assert_eq!(model._get_text("Sheet1!A2"), "#REF!");
}

#[test]
fn added_tables_replace_a_table_of_the_same_name() {
    let mut model = sales_model();
    // The table grows by one body row, without a totals row. The parser reads the tables when it parses a formula,
    // so the caller adds the tables before the formulas that use them.
    model.add_tables(vec![table(
        "SALES",
        "Sheet1",
        "A1:D6",
        &["Product", "Quantity", "Unit Price", "Amount"],
        0,
    )]);
    model._set("Sheet1!B5", "4");
    model._set("Sheet1!B6", "5");
    model._set("Sheet1!F1", "=SUM(Sales[Quantity])");
    model.evaluate();
    assert_eq!(model._get_text("Sheet1!F1"), "15");
    assert_eq!(model.workbook.tables.len(), 2);
    assert!(model.workbook.tables.contains_key("SALES"));
    assert!(!model.workbook.tables.contains_key("Sales"));
}

#[test]
fn an_optional_lambda_parameter_in_brackets_still_reads() {
    assert_eq!(
        value_at("Sheet1!F1", "=LAMBDA(a,[b],IF(ISOMITTED(b),a,a+b))(1)"),
        "1"
    );
    assert_eq!(
        value_at("Sheet1!F1", "=LAMBDA(a, [b], IF(ISOMITTED(b),a,a+b))(1,2)"),
        "3"
    );
}
