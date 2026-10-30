#![allow(clippy::unwrap_used)]

//! IronXL: a 3-D reference, such as `Data1:Data3!A1`, names the same cell or range on each sheet
//! from its first end to its last end, in the order of the sheets. The functions that Excel lets
//! take a 3-D reference read each sheet of the span. Every other place gives `#VALUE!`, and an end
//! that no sheet has gives `#REF!`.

use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::expressions::types::ParsedReference;
use crate::language::get_language;
use crate::locale::get_locale;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with the sheet `Sum` and then one sheet for each name of `sheets`, with `A1` of each
/// set to its position from 1. Then the `inputs` are set and the model is evaluated.
fn model_with<'a>(sheets: &[&str], inputs: &[(&str, &str)]) -> Model<'a> {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "Sum").unwrap();
    for (index, name) in sheets.iter().enumerate() {
        model.add_sheet(name).unwrap();
        model
            .set_user_input(index as u32 + 1, 1, 1, format!("{}", index + 1))
            .unwrap();
    }
    for (reference, input) in inputs {
        model._set(reference, input);
    }
    model.evaluate();
    model
}

/// The tokens of a formula, up to the end or the first illegal token.
fn tokens(formula: &str) -> Vec<TokenType> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mut lexer = Lexer::new(formula, LexerMode::A1, locale, language);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        let last = matches!(token, TokenType::EOF | TokenType::Illegal(_));
        tokens.push(token);
        if last {
            return tokens;
        }
    }
}

/// What `consume_three_d_reference` reads at the start of `formula`, and the text after it.
fn three_d(
    formula: &str,
    mode: LexerMode,
) -> Option<(
    String,
    String,
    ParsedReference,
    Option<ParsedReference>,
    String,
)> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mut lexer = Lexer::new(formula, mode, locale, language);
    let found = lexer.consume_three_d_reference();
    let rest: String = formula
        .chars()
        .skip(lexer.get_position() as usize)
        .collect();
    found.map(|(first, last, left, right)| (first, last, left, right, rest))
}

fn cell(column: i32, row: i32, absolute: bool) -> ParsedReference {
    ParsedReference {
        column,
        row,
        absolute_column: absolute,
        absolute_row: absolute,
    }
}

#[test]
fn each_form_of_a_3d_reference_sums_the_span() {
    let model = model_with(
        &["Data1", "Data2", "Data3"],
        &[
            ("Sum!A1", "=SUM(Data1:Data3!A1)"),
            ("Sum!A2", "=SUM(Data3:Data1!A1)"),
            ("Sum!A3", "=SUM(Data1:Data1!A1)"),
            ("Sum!A4", "=SUM(Data1:Data3!$A$1)"),
            ("Sum!A5", "=SUM(Data1:Data3!A$1)"),
            ("Sum!A6", "=SUM(data1:DATA3!A1)"),
            ("Sum!A7", "=SUM(Data1:Data3!A:A)"),
            ("Sum!A8", "=SUM(Data1:Data3!1:1)"),
            ("Sum!A9", "=SUM( Data1:Data3!A1 , 10)"),
            ("Sum!A10", "=SUM(Data2:Data3!A1)+SUM(Data1:Data2!A1)"),
        ],
    );
    assert_eq!(model._get_text("Sum!A1"), "6");
    assert_eq!(model._get_text("Sum!A2"), "6");
    assert_eq!(model._get_text("Sum!A3"), "1");
    assert_eq!(model._get_text("Sum!A4"), "6");
    assert_eq!(model._get_text("Sum!A5"), "6");
    assert_eq!(model._get_text("Sum!A6"), "6");
    assert_eq!(model._get_text("Sum!A7"), "6");
    assert_eq!(model._get_text("Sum!A8"), "6");
    assert_eq!(model._get_text("Sum!A9"), "16");
    assert_eq!(model._get_text("Sum!A10"), "8");
}

#[test]
fn ends_that_read_as_columns_and_quoted_ends_sum_the_span() {
    let model = model_with(
        &["Jan", "Feb", "Mar", "My Jan", "Mar's"],
        &[
            ("Sum!A1", "=SUM(Jan:Mar!A1)"),
            ("Sum!A2", "=SUM('My Jan:Mar''s'!A1)"),
            ("Sum!A3", "=SUM('Jan:Mar'!A1)"),
            ("Sum!A4", "=SUM(Mar:Jan!A1:B2)"),
        ],
    );
    assert_eq!(model._get_text("Sum!A1"), "6");
    assert_eq!(model._get_text("Sum!A2"), "9");
    assert_eq!(model._get_text("Sum!A3"), "6");
    assert_eq!(model._get_text("Sum!A4"), "6");
}

#[test]
fn a_relative_3d_reference_counts_from_its_cell() {
    let mut model = model_with(&["Data1", "Data2", "Data3"], &[]);
    for sheet in 1..=3 {
        model
            .set_user_input(sheet, 3, 2, format!("{}", sheet * 10))
            .unwrap();
    }
    model._set("Sum!A1", "=SUM(Data1:Data3!A1,Data1:Data3!$A$1)");
    model.evaluate();
    let filled = model.extend_to(0, 1, 1, 3, 2).unwrap();
    assert_eq!(filled, "=SUM(Data1:Data3!B3,Data1:Data3!$A$1)");
    model._set("Sum!B3", &filled);
    model.evaluate();
    assert_eq!(model._get_text("Sum!B3"), "66");
}

#[test]
fn a_3d_range_reads_every_cell_of_each_sheet() {
    let mut model = model_with(&["Data1", "Data2", "Data3"], &[]);
    // A1 = 1, 2, 3. B1 = 10, A2 = 100, B2 = 1000 on each sheet.
    for sheet in 1..=3 {
        model.set_user_input(sheet, 1, 2, "10".to_string()).unwrap();
        model
            .set_user_input(sheet, 2, 1, "100".to_string())
            .unwrap();
        model
            .set_user_input(sheet, 2, 2, "1000".to_string())
            .unwrap();
    }
    model._set("Sum!A1", "=SUM(Data1:Data3!A1:B2)");
    model._set("Sum!A2", "=COUNT(Data1:Data3!A1:B2)");
    model._set("Sum!A3", "=MAX(Data1:Data3!A1:B2)");
    model._set("Sum!A4", "=MIN(Data1:Data3!B2:A1)");
    model.evaluate();
    assert_eq!(model._get_text("Sum!A1"), "3336");
    assert_eq!(model._get_text("Sum!A2"), "12");
    assert_eq!(model._get_text("Sum!A3"), "1000");
    assert_eq!(model._get_text("Sum!A4"), "1");
}

/// Each function that takes a 3-D reference gives over the span the result that it gives for the
/// same range on each sheet, with numbers, text, Booleans, and empty cells on the sheets.
#[test]
fn each_listed_function_reads_the_span_with_its_range_rules() {
    let functions = [
        "SUM", "AVERAGE", "AVERAGEA", "COUNT", "COUNTA", "MAX", "MAXA", "MIN", "MINA", "PRODUCT",
        "STDEV", "STDEVA", "STDEVP", "STDEVPA", "STDEV.S", "STDEV.P", "VAR", "VARA", "VARP",
        "VARPA", "VAR.S", "VAR.P",
    ];
    let mut model = model_with(&["Data1", "Data2", "Data3"], &[]);
    // A1 = 1, 2, 3. B1 holds text, a Boolean, and a number as text. A2 is empty on Data2.
    let cells = [
        (1, "4", "abc", "-2"),
        (2, "", "TRUE", "7"),
        (3, "6", "'5", "0.5"),
    ];
    for (sheet, a2, b1, b2) in cells {
        model.set_user_input(sheet, 2, 1, a2.to_string()).unwrap();
        model.set_user_input(sheet, 1, 2, b1.to_string()).unwrap();
        model.set_user_input(sheet, 2, 2, b2.to_string()).unwrap();
    }
    for (row, name) in functions.iter().enumerate() {
        let row = row + 1;
        model._set(
            &format!("Sum!A{row}"),
            &format!("={name}(Data1:Data3!A1:B2)"),
        );
        model._set(
            &format!("Sum!B{row}"),
            &format!("={name}(Data1!A1:B2,Data2!A1:B2,Data3!A1:B2)"),
        );
        model._set(&format!("Sum!C{row}"), &format!("={name}(Data1:Data3!B1)"));
        model._set(
            &format!("Sum!D{row}"),
            &format!("={name}(Data1!B1:B1,Data2!B1:B1,Data3!B1:B1)"),
        );
    }
    model.evaluate();
    for (row, name) in functions.iter().enumerate() {
        let row = row + 1;
        for (three_d, sheets) in [("A", "B"), ("C", "D")] {
            let actual = model._get_text(&format!("Sum!{three_d}{row}"));
            let expected = model._get_text(&format!("Sum!{sheets}{row}"));
            assert_eq!(actual, expected, "{name} in row {row}");
            assert!(
                !actual.starts_with("#VALUE"),
                "{name} in row {row}: {actual}"
            );
        }
    }
    // Fixed values of a few functions over the span A1:B2.
    assert_eq!(model._get_text("Sum!A1"), "21.5");
    assert_eq!(model._get_text("Sum!A4"), "8");
    assert_eq!(model._get_text("Sum!A5"), "11");
    assert_eq!(model._get_text("Sum!A6"), "7");
    assert_eq!(model._get_text("Sum!A8"), "-2");
    assert_eq!(model._get_text("Sum!A10"), "-1008");
    // B1 holds text, TRUE, and a number as text. COUNT and MAX read no number in them. COUNTA reads
    // three values. MAXA and MINA read TRUE as 1 and text as 0, as for a range.
    assert_eq!(model._get_text("Sum!C4"), "0");
    assert_eq!(model._get_text("Sum!C5"), "3");
    assert_eq!(model._get_text("Sum!C6"), "0");
    assert_eq!(model._get_text("Sum!C7"), "1");
    assert_eq!(model._get_text("Sum!C9"), "0");
}

#[test]
fn a_3d_reference_outside_a_listed_function_gives_value_error() {
    let model = model_with(
        &["Data1", "Data2", "Data3"],
        &[
            ("Sum!A1", "=Data1:Data3!A1"),
            ("Sum!A2", "=VLOOKUP(1,Data1:Data3!A1:B2,2,FALSE)"),
            ("Sum!A3", "=@Data1:Data3!A1"),
            ("Sum!A4", "=ABS(Data1:Data3!A1)"),
            ("Sum!A5", "=Data1:Data3!A1+1"),
            ("Sum!A6", "=SUMIF(Data1:Data3!A1,\">0\")"),
            ("Sum!A7", "=SUM(Data1:Data3!A1*2)"),
            ("Sum!A8", "=Data1:Data1!A1"),
            ("Sum!A9", "='Data1:Data3'!A1"),
            ("Sum!A10", "=ABS('Data1:Data3'!A1)"),
        ],
    );
    for reference in [
        "Sum!A1", "Sum!A2", "Sum!A3", "Sum!A4", "Sum!A5", "Sum!A6", "Sum!A7", "Sum!A8", "Sum!A9",
        "Sum!A10",
    ] {
        assert_eq!(model._get_text(reference), "#VALUE!", "{reference}");
    }
}

#[test]
fn an_end_that_is_not_a_sheet_gives_ref_error() {
    let model = model_with(
        &["Data1", "Data2", "Data3"],
        &[
            ("Sum!A1", "=SUM(Data1:Nope!A1)"),
            ("Sum!A2", "=SUM(Nope:Data3!A1)"),
            ("Sum!A3", "=COUNT(Nope:Data3!A1)"),
            ("Sum!A4", "=Nope:Data3!A1"),
            ("Sum!A5", "=SUM('Data1:No pe'!A1)"),
            ("Sum!A6", "=SUM(Data1:Data3!A1,Data1:Nope!A1)"),
        ],
    );
    for reference in ["Sum!A1", "Sum!A2", "Sum!A3", "Sum!A4", "Sum!A5", "Sum!A6"] {
        assert_eq!(model._get_text(reference), "#REF!", "{reference}");
    }
}

#[test]
fn the_span_follows_the_order_of_the_sheets() {
    let mut model = model_with(
        &["Data1", "Data2", "Data3"],
        &[("Sum!A1", "=SUM(Data1:Data3!A1)")],
    );
    assert_eq!(model._get_text("Sum!A1"), "6");
    // Data1, Data3, Data2: Data2 is outside the span.
    model.move_sheet(2, 3).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("Sum!A1"), "4");
    model.move_sheet(3, 2).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("Sum!A1"), "6");
    // Without Data2.
    model.delete_sheet(2).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("Sum!A1"), "4");
    // A new sheet between Data1 and Data3 is in the span.
    model.insert_sheet("New", 2, None).unwrap();
    model.set_user_input(2, 1, 1, "10".to_string()).unwrap();
    model.evaluate();
    assert_eq!(model._get_text("Sum!A1"), "14");
}

#[test]
fn a_formula_in_the_span_evaluates_before_the_sum() {
    let model = model_with(
        &["Data1", "Data2", "Data3"],
        &[
            ("Data2!A1", "=Data3!A1*10"),
            ("Sum!A1", "=SUM(Data1:Data3!A1)"),
            ("Sum!A2", "=Sum!A1+1"),
        ],
    );
    assert_eq!(model._get_text("Data2!A1"), "30");
    assert_eq!(model._get_text("Sum!A1"), "34");
    assert_eq!(model._get_text("Sum!A2"), "35");
}

#[test]
fn a_3d_reference_keeps_its_text() {
    let model = model_with(
        &["Data1", "Data2", "Data3", "Jan", "Mar", "My Jan", "Mar's"],
        &[
            ("Sum!A1", "=SUM(Data1:Data3!A1)"),
            ("Sum!A2", "=SUM(Data3:Data1!$A$1:B$2)"),
            ("Sum!A3", "=SUM(Jan:Mar!A:A)"),
            ("Sum!A4", "=SUM('My Jan:Mar''s'!A1)"),
            ("Sum!A5", "=SUM(data1:DATA3!1:1)"),
            ("Sum!A6", "=Data1:Data3!A1"),
        ],
    );
    assert_eq!(model._get_formula("Sum!A1"), "=SUM(Data1:Data3!A1)");
    assert_eq!(model._get_formula("Sum!A2"), "=SUM(Data3:Data1!$A$1:B$2)");
    assert_eq!(model._get_formula("Sum!A3"), "=SUM(Jan:Mar!A:A)");
    assert_eq!(model._get_formula("Sum!A4"), "=SUM('My Jan:Mar''s'!A1)");
    assert_eq!(model._get_formula("Sum!A5"), "=SUM(data1:DATA3!1:1)");
    assert_eq!(model._get_formula("Sum!A6"), "=Data1:Data3!A1");
}

#[test]
fn a_rename_of_an_end_changes_the_3d_reference() {
    let mut model = model_with(
        &["Data1", "Data2", "Data3"],
        &[("Sum!A1", "=SUM(Data1:Data3!A1)")],
    );
    model.rename_sheet("Data3", "Last Data").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Sum!A1"), "=SUM('Data1:Last Data'!A1)");
    assert_eq!(model._get_text("Sum!A1"), "6");
    model.rename_sheet("Data1", "First").unwrap();
    model.rename_sheet("Last Data", "Last").unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Sum!A1"), "=SUM(First:Last!A1)");
    assert_eq!(model._get_text("Sum!A1"), "6");
}

#[test]
fn a_cut_and_paste_keeps_a_3d_reference() {
    let mut model = model_with(
        &["Data1", "Data2", "Data3"],
        &[("Sum!A1", "=SUM(Data1:Data3!A1)")],
    );
    let source = crate::expressions::types::CellReferenceIndex {
        sheet: 0,
        row: 1,
        column: 1,
    };
    let target = crate::expressions::types::CellReferenceIndex {
        sheet: 0,
        row: 5,
        column: 3,
    };
    let area = crate::expressions::types::Area {
        sheet: 0,
        row: 1,
        column: 1,
        width: 1,
        height: 1,
    };
    let moved = model
        .move_cell_value_to_area("=SUM(Data1:Data3!A1)+A1", &source, &target, &area)
        .unwrap();
    assert_eq!(moved, "=SUM(Data1:Data3!A1)+C5");
    model._set("Sum!C5", &moved.replace("+C5", ""));
    model.evaluate();
    assert_eq!(model._get_formula("Sum!C5"), "=SUM(Data1:Data3!A1)");
    assert_eq!(model._get_text("Sum!C5"), "6");
}

#[test]
fn a_3d_reference_to_another_workbook_does_not_change() {
    let model = model_with(
        &["Data1", "Data2", "Data3"],
        &[("Sum!A1", "=SUM([1]Data1:Data3!A1)")],
    );
    assert_eq!(model._get_text("Sum!A1"), "#ERROR!");
}

#[test]
fn the_lexer_reads_a_3d_reference_only_on_request() {
    // The token stream does not change.
    assert_eq!(
        tokens("Data1:Data3!A1"),
        vec![
            TokenType::Ident("Data1".to_string()),
            TokenType::Colon,
            TokenType::Reference {
                sheet: Some("Data3".to_string()),
                row: 1,
                column: 1,
                absolute_column: false,
                absolute_row: false,
            },
            TokenType::EOF,
        ]
    );
    assert!(matches!(
        tokens("Jan:Mar!A1").as_slice(),
        [
            TokenType::Range { sheet: None, .. },
            TokenType::Bang,
            TokenType::Reference { sheet: None, .. },
            TokenType::EOF
        ]
    ));
    assert!(matches!(
        tokens("'My Jan:Mar'!A1").as_slice(),
        [TokenType::Reference { sheet: Some(sheet), .. }, TokenType::EOF] if sheet == "My Jan:Mar"
    ));
    // The reader of a 3-D reference.
    assert_eq!(
        three_d(" Data1:Data3!$A$1:B2)", LexerMode::A1),
        Some((
            "Data1".to_string(),
            "Data3".to_string(),
            cell(1, 1, true),
            Some(cell(2, 2, false)),
            ")".to_string()
        ))
    );
    assert_eq!(
        three_d("'My Jan:Mar''s'!A1", LexerMode::A1),
        Some((
            "My Jan".to_string(),
            "Mar's".to_string(),
            cell(1, 1, false),
            None,
            "".to_string()
        ))
    );
    assert_eq!(
        three_d("Data1:Data3!R1C1", LexerMode::R1C1),
        Some((
            "Data1".to_string(),
            "Data3".to_string(),
            cell(1, 1, true),
            None,
            "".to_string()
        ))
    );
    for other in [
        "Data1!A1",
        "Data1:Data3",
        "Data1:Data3!Rate",
        "A1:B2!C3",
        "Data1:'Data 3'!A1",
        "'Data1'!A1",
        "'Data1:'!A1",
        "':Data3'!A1",
        "'[1]Data1:Data3'!A1",
        "'Data1:Data3",
        "SUM(A1)",
        "",
    ] {
        assert_eq!(three_d(other, LexerMode::A1), None, "{other}");
    }
}

#[test]
fn a_3d_reference_after_a_bracket_is_in_another_workbook() {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mut lexer = Lexer::new("[1]Data1:Data3!A1", LexerMode::A1, locale, language);
    for _ in 0..3 {
        lexer.next_token();
    }
    assert_eq!(lexer.consume_three_d_reference(), None);
    // Nothing moved.
    assert_eq!(lexer.get_position(), 3);
}
