#![allow(clippy::unwrap_used)]

// IronXL: Excel reads the code letters of a number format (Y, M, D, H, S, E, General, AM/PM,
// and A/P) in any letter case. Text in quotes and after `\` keeps its case.

use crate::{
    formatter::{
        format::format_number,
        lexer::{is_likely_date_number_format, Lexer, Token},
        parser::Parser,
    },
    locale::{get_locale, Locale},
    test::util::new_empty_model,
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

// 2023-03-15 (a Wednesday) 13:05:04
const AFTERNOON: f64 = 45000.0 + 47104.0 / 86400.0;
// 2023-03-15 01:30:00
const MORNING: f64 = 45000.0 + 5400.0 / 86400.0;

fn tokens(format: &str) -> Vec<Token> {
    let mut lexer = Lexer::new(format);
    let mut tokens = vec![];
    loop {
        let token = lexer.next_token();
        if token == Token::EOF {
            return tokens;
        }
        tokens.push(token);
    }
}

fn parses(format: &str) -> bool {
    let mut parser = Parser::new(format);
    parser.parse();
    !parser.parts.is_empty() && parser.parts.iter().all(|part| !part.is_error())
}

fn text(value: f64, format: &str) -> String {
    format_number(value, format, get_default_locale()).text
}

#[test]
fn upper_and_mixed_case_codes_give_the_tokens_of_the_lowercase_code() {
    let codes = [
        ("MM:SS", "mm:ss"),
        ("H:MM", "h:mm"),
        ("[H]:MM:SS", "[h]:mm:ss"),
        ("YYYY-MM-DD", "yyyy-mm-dd"),
        ("DD/MM/YYYY", "dd/mm/yyyy"),
        ("0.00e+00", "0.00E+00"),
        ("0.00e-00", "0.00E-00"),
        ("GENERAL", "General"),
        ("gEnErAl", "General"),
        ("Mm:Ss", "mm:ss"),
        ("Yyyy-Mm-Dd", "yyyy-mm-dd"),
        ("yYyY-mM-dD", "yyyy-mm-dd"),
        ("YY", "yy"),
        ("D", "d"),
        ("DDD", "ddd"),
        ("DDDD", "dddd"),
        ("MMM", "mmm"),
        ("MMMM", "mmmm"),
        ("MMMMM", "mmmmm"),
        ("SS", "ss"),
        ("H:MM:SS AM/PM", "h:mm:ss AM/PM"),
        ("h:mm am/pm", "h:mm AM/PM"),
    ];
    for (code, lower) in codes {
        assert_eq!(tokens(code), tokens(lower), "{code}");
        assert!(parses(code), "{code}");
    }
}

#[test]
fn upper_and_mixed_case_codes_give_the_text_of_the_lowercase_code() {
    let cases = [
        (AFTERNOON, "YYYY-MM-DD", "yyyy-mm-dd", "2023-03-15"),
        (AFTERNOON, "Yyyy-Mm-Dd", "yyyy-mm-dd", "2023-03-15"),
        (AFTERNOON, "DD/MM/YYYY", "dd/mm/yyyy", "15/03/2023"),
        (AFTERNOON, "YYYY-MM", "yyyy-mm", "2023-03"),
        (AFTERNOON, "YY", "yy", "23"),
        (AFTERNOON, "D", "d", "15"),
        (AFTERNOON, "DDD", "ddd", "Wed"),
        (AFTERNOON, "DDDD", "dddd", "Wednesday"),
        (AFTERNOON, "MMM", "mmm", "Mar"),
        (AFTERNOON, "MMMM", "mmmm", "March"),
        (AFTERNOON, "MMMMM", "mmmmm", "M"),
        (AFTERNOON, "SS", "ss", "04"),
        (AFTERNOON, "H:MM", "h:mm", "13:05"),
        (AFTERNOON, "MM:SS", "mm:ss", "05:04"),
        (AFTERNOON, "Mm:Ss", "mm:ss", "05:04"),
        (AFTERNOON, "[H]:MM:SS", "[h]:mm:ss", "1080013:05:04"),
        (AFTERNOON, "H:MM:SS AM/PM", "h:mm:ss AM/PM", "1:05:04 PM"),
        (AFTERNOON, "h:mm am/pm", "h:mm AM/PM", "1:05 PM"),
        (1234.5, "0.00e+00", "0.00E+00", "1.23E+03"),
        (0.001234, "0.00e-00", "0.00E-00", "1.23E-03"),
        (1234.5, "GENERAL", "General", "1234.5"),
        (1234.5, "gEnErAl", "General", "1234.5"),
    ];
    for (value, code, lower, expected) in cases {
        assert_eq!(text(value, code), expected, "{code}");
        assert_eq!(text(value, lower), expected, "{lower}");
    }
}

#[test]
fn m_is_a_minute_after_an_hour_and_before_a_second_in_both_cases() {
    let cases = [
        // after an hour
        ("h:m", "13:5"),
        ("H:M", "13:5"),
        ("hh:mm", "13:05"),
        ("HH:MM", "13:05"),
        // before a second, with literals or nothing between
        ("m:ss", "5:04"),
        ("M:SS", "5:04"),
        ("mm:ss", "05:04"),
        ("MM:SS", "05:04"),
        ("mm ss", "05 04"),
        ("MMSS", "0504"),
        // otherwise a month
        ("mm", "03"),
        ("MM", "03"),
        ("m/d", "3/15"),
        ("M/D", "3/15"),
        ("dd mm", "15 03"),
        ("DD MM", "15 03"),
        ("mm \"ss\"", "03 ss"),
        ("yyyy-mm-dd hh:mm:ss", "2023-03-15 13:05:04"),
        ("YYYY-MM-DD HH:MM:SS", "2023-03-15 13:05:04"),
    ];
    for (code, expected) in cases {
        assert_eq!(text(AFTERNOON, code), expected, "{code}");
    }
}

#[test]
fn a_over_p_shows_the_half_of_the_day_in_the_case_of_the_code() {
    assert_eq!(tokens("A/P"), vec![Token::AP(false)]);
    assert_eq!(tokens("a/p"), vec![Token::AP(true)]);
    assert_eq!(tokens("A/p"), vec![Token::AP(false)]);
    assert_eq!(tokens("a/P"), vec![Token::AP(true)]);
    assert!(parses("h:mm A/P"));
    assert!(parses("h:mm a/p"));
    assert_eq!(text(AFTERNOON, "h:mm A/P"), "1:05 P");
    assert_eq!(text(AFTERNOON, "h:mm a/p"), "1:05 p");
    assert_eq!(text(MORNING, "h:mm A/P"), "1:30 A");
    assert_eq!(text(MORNING, "h:mm a/p"), "1:30 a");
    assert_eq!(text(45000.0, "h:mm A/P"), "12:00 A");
    assert_eq!(text(45000.5, "hh:mm a/p"), "12:00 p");
}

#[test]
fn quoted_and_escaped_letters_keep_their_case() {
    assert_eq!(
        tokens("\"YYYY\"0"),
        vec![Token::Text("YYYY".into()), Token::Zero]
    );
    assert_eq!(
        tokens("\\Y\\M0"),
        vec![Token::Literal('Y'), Token::Literal('M'), Token::Zero]
    );
    assert!(parses("\"YYYY\"0"));
    assert!(parses("\\Y\\M0"));
    assert_eq!(text(7.0, "\"YYYY\"0"), "YYYY7");
    assert_eq!(text(7.0, "\\Y\\M0"), "YM7");
    assert_eq!(text(7.0, "\\y\\m0"), "ym7");
}

#[test]
fn codes_that_are_not_valid_stay_refused() {
    let codes = [
        "x",
        "Q0",
        "[Blu]0.00",
        "0.00E",
        "0.00e",
        "0.00eX",
        "0.00EX",
        "HHH",
        "hhh",
        "SSS",
        "sss",
        "GENERA",
        "GENERAX",
        "A",
        "a",
        "A/",
        "a/x",
        "A/X",
        "AM",
        "AM/",
        "AM/P",
        "AM/PX",
    ];
    for code in codes {
        assert!(!parses(code), "{code}");
    }
}

#[test]
fn uppercase_date_codes_are_likely_dates() {
    for code in ["YYYY-MM-DD", "DD/MM/YYYY", "MMM", "Yy", "h:mm A/P", "dD"] {
        assert!(is_likely_date_number_format(code), "{code}");
    }
    for code in ["0.00e+00", "GENERAL", "\"YYYY\"0", "\\D0"] {
        assert!(!is_likely_date_number_format(code), "{code}");
    }
}

#[test]
fn text_function_reads_uppercase_codes() {
    let mut model = new_empty_model();
    model._set("A1", "45000.5");
    model._set("B1", "=TEXT(A1,\"YYYY-MM-DD\")");
    model._set("B2", "=TEXT(A1,\"yyyy-mm-dd\")");
    model._set("B3", "=TEXT(A1,\"H:MM\")");
    model._set("B4", "=TEXT(A1,\"MM:SS\")");
    model._set("B5", "=TEXT(1234.5,\"0.00e+00\")");
    model._set("B6", "=TEXT(1234.5,\"GENERAL\")");
    model._set("B7", "=TEXT(A1,\"h:mm a/p\")");
    model.evaluate();
    assert_eq!(model._get_text("B1"), "2023-03-15");
    assert_eq!(model._get_text("B2"), "2023-03-15");
    assert_eq!(model._get_text("B3"), "12:00");
    assert_eq!(model._get_text("B4"), "00:00");
    assert_eq!(model._get_text("B5"), "1.23E+03");
    assert_eq!(model._get_text("B6"), "1234.5");
    assert_eq!(model._get_text("B7"), "12:00 p");
}
