#![allow(clippy::unwrap_used)]

use crate::expressions::{
    lexer::{Lexer, LexerMode},
    token::{TableReference, TableSpecifier, TokenType::*},
};
use crate::language::get_language;
use crate::locale::get_locale;

fn new_lexer(formula: &str) -> Lexer<'_> {
    let locale = get_locale("en").unwrap();
    let language = get_language("en").unwrap();
    Lexer::new(formula, LexerMode::A1, locale, language)
}

#[test]
fn table_this_row() {
    let mut lx = new_lexer("tbInfo[[#This Row], [Jan]:[Dec]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::ThisRow),
            table_reference: Some(TableReference::RangeReference((
                "Jan".to_string(),
                "Dec".to_string()
            )))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier() {
    let mut lx = new_lexer("tbInfo[December]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("December".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_no_specifier_white_spaces() {
    let mut lx = new_lexer("tbInfo[[First Month]]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: None,
            table_reference: Some(TableReference::ColumnReference("First Month".to_string()))
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_totals_no_reference() {
    let mut lx = new_lexer("tbInfo[#Totals]");
    assert_eq!(
        lx.next_token(),
        StructuredReference {
            table_name: "tbInfo".to_string(),
            specifier: Some(TableSpecifier::Totals),
            table_reference: None
        }
    );
    assert_eq!(lx.next_token(), EOF);
}

/// The token of a structured reference.
fn reference(
    table_name: &str,
    specifier: Option<TableSpecifier>,
    table_reference: Option<TableReference>,
) -> crate::expressions::token::TokenType {
    StructuredReference {
        table_name: table_name.to_string(),
        specifier,
        table_reference,
    }
}

fn column(name: &str) -> Option<TableReference> {
    Some(TableReference::ColumnReference(name.to_string()))
}

#[test]
fn table_whole_body() {
    let mut lx = new_lexer("tbInfo[]");
    assert_eq!(lx.next_token(), reference("tbInfo", None, None));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_this_row_at() {
    let mut lx = new_lexer("tbInfo[@Jan]+tbInfo[@[First Month]]*tbInfo[@[Jan]:[Dec]]+tbInfo[@]");
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::ThisRow), column("Jan"))
    );
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference(
            "tbInfo",
            Some(TableSpecifier::ThisRow),
            column("First Month")
        )
    );
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference(
            "tbInfo",
            Some(TableSpecifier::ThisRow),
            Some(TableReference::RangeReference((
                "Jan".to_string(),
                "Dec".to_string()
            )))
        )
    );
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::ThisRow), None)
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_without_a_table_name() {
    let mut lx = new_lexer("[@Quantity]*[Unit Price]+SUM([[#Data],[Jan]])");
    assert_eq!(
        lx.next_token(),
        reference("", Some(TableSpecifier::ThisRow), column("Quantity"))
    );
    lx.next_token();
    assert_eq!(lx.next_token(), reference("", None, column("Unit Price")));
    lx.next_token();
    lx.next_token();
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference("", Some(TableSpecifier::Data), column("Jan"))
    );
    assert_eq!(lx.next_token(), RightParenthesis);
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn brackets_of_another_workbook_are_not_a_structured_reference() {
    // [1]Sheet1!A1, [Book.xlsx]Sheet1!A1, [1]!Rate, [1], and []
    for formula in ["[1]Sheet1!A1", "[Book]Sheet1!A1", "[1]!Rate", "[1]", "[]"] {
        let mut lx = new_lexer(formula);
        assert_eq!(lx.next_token(), LeftBracket, "{formula}");
    }
}

#[test]
fn table_two_specifiers() {
    let mut lx = new_lexer("tbInfo[[#Headers],[#Data]]+tbInfo[[#Data], [#Totals], [Jan]]");
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::HeadersData), None)
    );
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::DataTotals), column("Jan"))
    );
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_specifier_without_case() {
    let mut lx = new_lexer("tbInfo[#totals]+tbInfo[[#this row],[Jan]]");
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::Totals), None)
    );
    lx.next_token();
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", Some(TableSpecifier::ThisRow), column("Jan"))
    );
}

#[test]
fn table_escaped_column_name() {
    let mut lx = new_lexer("tbInfo[a''b'[c']'#d'@]+tbInfo[[x'[1']]]");
    assert_eq!(
        lx.next_token(),
        reference("tbInfo", None, column("a'b[c]#d@"))
    );
    lx.next_token();
    assert_eq!(lx.next_token(), reference("tbInfo", None, column("x[1]")));
    assert_eq!(lx.next_token(), EOF);
}

#[test]
fn table_invalid_references() {
    for formula in [
        "tbInfo[[#Data],[#Headers]]",
        "tbInfo[[#All],[#Data]]",
        "tbInfo[#Nope]",
        "tbInfo[[Jan],[#Data]]",
        "tbInfo[[Jan],[Dec]]",
        "tbInfo[[Jan]",
        "tbInfo[Jan'",
        "tbInfo[[]]",
    ] {
        let mut lx = new_lexer(formula);
        assert!(matches!(lx.next_token(), Illegal(_)), "{formula}");
    }
}
