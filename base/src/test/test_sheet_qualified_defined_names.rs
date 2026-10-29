#![allow(clippy::unwrap_used)]

//! IronXL: a formula can name a defined name of a sheet with its sheet, such as `One!Local` or
//! `'My Sheet'!Local`. It resolves to the name of that sheet from a formula on any sheet, as in
//! Excel. It never resolves to a workbook name. A missing name or a missing sheet gives `#NAME?`.

use crate::cell::CellValue;
use crate::expressions::lexer::{Lexer, LexerMode};
use crate::expressions::token::TokenType;
use crate::language::get_language;
use crate::locale::get_locale;
use crate::model::Model;
use crate::test::util::new_empty_model;

/// A model with the sheets `One`, `Two`, and `My Sheet`, the `names` added, then the `inputs`
/// set, then evaluated.
fn model_with<'a>(names: &[(&str, Option<u32>, &str)], inputs: &[(&str, &str)]) -> Model<'a> {
    let mut model = new_empty_model();
    model.rename_sheet("Sheet1", "One").unwrap();
    model.add_sheet("Two").unwrap();
    model.add_sheet("My Sheet").unwrap();
    for result in model.add_defined_names(names) {
        result.unwrap();
    }
    for (reference, input) in inputs {
        model._set(reference, input);
    }
    model.evaluate();
    model
}

/// The number in a cell, or NaN for any other value, so a comparison with a number fails.
fn number(model: &Model, reference: &str) -> f64 {
    match model.get_cell_value_by_ref(reference).unwrap() {
        CellValue::Number(number) => number,
        _ => f64::NAN,
    }
}

/// The tokens of a formula in the mode `mode`, up to the end or the first illegal token.
fn tokens(formula: &str, mode: &LexerMode) -> Vec<TokenType> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    let mut lexer = Lexer::new(formula, mode.clone(), locale, language);
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

fn sheet_name(sheet: &str, name: &str) -> TokenType {
    TokenType::SheetDefinedName {
        sheet: sheet.to_string(),
        name: name.to_string(),
    }
}

#[test]
fn a_sheet_qualified_name_resolves_from_another_sheet() {
    let model = model_with(
        &[("Local", Some(0), "5"), ("Spaced", Some(2), "7")],
        &[
            ("Two!A1", "=One!Local*2"),
            ("Two!A2", "='One'!Local*2"),
            ("Two!A3", "=one!LOCAL*2"),
            ("Two!A4", "='My Sheet'!Spaced*2"),
            ("One!A1", "=One!Local+1"),
            ("Two!A5", "=SUM(One!Local,'My Sheet'!Spaced)"),
        ],
    );
    assert_eq!(number(&model, "Two!A1"), 10.0);
    assert_eq!(number(&model, "Two!A2"), 10.0);
    assert_eq!(number(&model, "Two!A3"), 10.0);
    assert_eq!(number(&model, "Two!A4"), 14.0);
    assert_eq!(number(&model, "One!A1"), 6.0);
    assert_eq!(number(&model, "Two!A5"), 12.0);
    // The formula keeps the sheet and the name as written. The sheet gets quotes only when it needs them.
    assert_eq!(model._get_formula("Two!A1"), "=One!Local*2");
    assert_eq!(model._get_formula("Two!A2"), "=One!Local*2");
    assert_eq!(model._get_formula("Two!A3"), "=one!LOCAL*2");
    assert_eq!(model._get_formula("Two!A4"), "='My Sheet'!Spaced*2");
}

#[test]
fn a_missing_name_or_a_missing_sheet_gives_name_error() {
    let model = model_with(
        &[("Local", Some(0), "5")],
        &[
            ("Two!A1", "=Two!Local"),
            ("Two!A2", "=Missing!Local"),
            ("Two!A3", "=One!Other"),
            ("Two!A4", "='No Sheet'!Local*2"),
        ],
    );
    for cell in ["Two!A1", "Two!A2", "Two!A3", "Two!A4"] {
        assert_eq!(model._get_text(cell), "#NAME?", "{cell}");
    }
    assert_eq!(model._get_formula("Two!A2"), "=Missing!Local");
    assert_eq!(model._get_formula("Two!A4"), "='No Sheet'!Local*2");
}

#[test]
fn a_workbook_name_does_not_resolve_a_sheet_qualified_name() {
    let model = model_with(
        &[("Local", None, "5")],
        &[("Two!A1", "=One!Local"), ("Two!A2", "=Local")],
    );
    assert_eq!(model._get_text("Two!A1"), "#NAME?");
    assert_eq!(number(&model, "Two!A2"), 5.0);
}

#[test]
fn a_sheet_name_and_a_workbook_name_of_the_same_text_each_give_their_own_value() {
    let model = model_with(
        &[("Local", Some(0), "5"), ("Local", None, "7")],
        &[
            ("Two!A1", "=One!Local"),
            ("Two!A2", "=Local"),
            ("One!A1", "=Local"),
            ("One!A2", "=One!Local"),
        ],
    );
    assert_eq!(number(&model, "Two!A1"), 5.0);
    assert_eq!(number(&model, "Two!A2"), 7.0);
    assert_eq!(number(&model, "One!A1"), 5.0);
    assert_eq!(number(&model, "One!A2"), 5.0);
}

#[test]
fn a_sheet_qualified_reference_name_follows_its_cells() {
    let mut model = model_with(
        &[
            ("Cell", Some(0), "One!$A$1"),
            ("Column", Some(0), "One!$A$1:$A$3"),
        ],
        &[
            ("One!A1", "2"),
            ("One!A2", "3"),
            ("One!A3", "4"),
            ("Two!A1", "=One!Cell*10"),
            ("Two!A2", "=SUM(One!Column)"),
        ],
    );
    assert_eq!(number(&model, "Two!A1"), 20.0);
    assert_eq!(number(&model, "Two!A2"), 9.0);
    model._set("One!A1", "5");
    model.evaluate();
    assert_eq!(number(&model, "Two!A1"), 50.0);
    assert_eq!(number(&model, "Two!A2"), 12.0);
}

#[test]
fn a_rename_of_the_sheet_or_the_name_changes_the_formula() {
    let mut model = model_with(
        &[("Local", Some(0), "5"), ("Local", Some(2), "1")],
        &[("Two!A1", "=One!Local*2"), ("Two!A2", "='My Sheet'!Local")],
    );
    model.rename_sheet("One", "First Sheet").unwrap();
    assert_eq!(model._get_formula("Two!A1"), "='First Sheet'!Local*2");
    assert_eq!(model._get_formula("Two!A2"), "='My Sheet'!Local");
    model
        .update_defined_name("Local", Some(0), "Rate", Some(0), "6")
        .unwrap();
    model.evaluate();
    assert_eq!(model._get_formula("Two!A1"), "='First Sheet'!Rate*2");
    // The name of another sheet with the same text stays.
    assert_eq!(model._get_formula("Two!A2"), "='My Sheet'!Local");
    assert_eq!(number(&model, "Two!A1"), 12.0);
    assert_eq!(number(&model, "Two!A2"), 1.0);
}

#[test]
fn an_extended_formula_keeps_the_sheet_qualified_name() {
    let model = model_with(&[("Local", Some(0), "5")], &[("Two!A1", "=One!Local*B1")]);
    // Two!A1 to Two!A3
    let result = model.extend_to(1, 1, 1, 3, 1).unwrap();
    assert_eq!(result, "=One!Local*B3");
}

#[test]
fn the_lexer_reads_a_sheet_qualified_name_as_one_token() {
    for mode in &[LexerMode::A1, LexerMode::R1C1] {
        assert_eq!(
            tokens("Sheet1!Rate*2", mode)[..2],
            [
                sheet_name("Sheet1", "Rate"),
                TokenType::Product(crate::expressions::token::OpProduct::Times)
            ]
        );
        assert_eq!(
            tokens("'My Sheet'!Rate", mode),
            [sheet_name("My Sheet", "Rate"), TokenType::EOF]
        );
        assert_eq!(
            tokens("Sheet1!Cost_2.x", mode),
            [sheet_name("Sheet1", "Cost_2.x"), TokenType::EOF]
        );
        assert_eq!(
            tokens("Sheet1!Rate1", mode),
            [sheet_name("Sheet1", "Rate1"), TokenType::EOF]
        );
    }
}

#[test]
fn the_lexer_reads_references_errors_and_external_names_as_before() {
    let a1 = |formula| tokens(formula, &LexerMode::A1);
    assert!(matches!(
        a1("Sheet1!A1")[0],
        TokenType::Reference {
            sheet: Some(_),
            row: 1,
            column: 1,
            ..
        }
    ));
    assert!(matches!(
        a1("Sheet1!$B$2")[0],
        TokenType::Reference {
            row: 2,
            column: 2,
            ..
        }
    ));
    assert!(matches!(a1("Sheet1!A:A")[0], TokenType::Range { .. }));
    assert!(matches!(a1("Sheet1!1:1")[0], TokenType::Range { .. }));
    assert!(matches!(a1("Sheet1!A1:B2")[0], TokenType::Range { .. }));
    assert!(matches!(a1("'My Sheet'!A1:B2")[0], TokenType::Range { .. }));
    assert!(matches!(a1("Sheet1!#REF!")[0], TokenType::Illegal(_)));
    assert!(matches!(
        a1("Sheet1!R1C1")[0],
        TokenType::Reference { row: 1, .. }
    ));
    assert!(matches!(
        tokens("Sheet1!R1C1", &LexerMode::R1C1)[0],
        TokenType::Reference {
            row: 1,
            column: 1,
            ..
        }
    ));
    // A structured reference, a function call, and a second '!' after a sheet stay illegal.
    assert!(matches!(
        a1("Sheet1!Table1[Price]")[0],
        TokenType::Illegal(_)
    ));
    assert!(matches!(a1("Sheet1!Square(2)")[0], TokenType::Illegal(_)));
    assert!(matches!(a1("Sheet1!Rate!A1")[0], TokenType::Illegal(_)));
    // A name of another workbook: [1]!Rate
    assert_eq!(
        a1("[1]!Rate"),
        [
            TokenType::LeftBracket,
            TokenType::Number(1.0),
            TokenType::RightBracket,
            TokenType::Bang,
            TokenType::Ident("Rate".to_string()),
            TokenType::EOF
        ]
    );
}

#[test]
fn references_with_a_sheet_evaluate_as_before() {
    let model = model_with(
        &[],
        &[
            ("One!A1", "2"),
            ("One!B2", "3"),
            ("Two!A1", "=One!A1"),
            ("Two!A2", "=SUM(One!A:A)"),
            ("Two!A3", "=SUM(One!1:1)"),
            ("Two!A4", "=SUM(One!A1:B2)"),
            ("Two!A5", "=SUM('My Sheet'!A1:B2)"),
        ],
    );
    assert_eq!(number(&model, "Two!A1"), 2.0);
    assert_eq!(number(&model, "Two!A2"), 2.0);
    assert_eq!(number(&model, "Two!A3"), 2.0);
    assert_eq!(number(&model, "Two!A4"), 5.0);
    assert_eq!(number(&model, "Two!A5"), 0.0);
}
