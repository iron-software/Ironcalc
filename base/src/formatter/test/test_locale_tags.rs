#![allow(clippy::unwrap_used)]

// IronXL: a locale tag without a currency symbol, such as `[$-409]`, goes before the parse.
// A system tag gives the en-US code of the system date or time.

use crate::formatter::format::{system_code, text_function_code, without_locale_tags};

#[test]
fn without_locale_tags_removes_each_tag_outside_quoted_text() {
    let cases = [
        ("[$-409]h:mm:ss", "h:mm:ss"),
        ("[$-F400]h:mm:ss AM/PM", "h:mm:ss AM/PM"),
        ("[$-en-US]0.00", "0.00"),
        ("0.00;[$-409]-0.00", "0.00;-0.00"),
        ("[$-409][$-409]0", "0"),
        ("ss[$-409].0", "ss\"\".0"),
        ("[$-409].00", "\"\".00"),
        ("\"[$-409]\"0", "\"[$-409]\"0"),
        ("\\[$-409]0", "\\[$-409]0"),
        ("\\\u{e9}0", "\\\u{e9}0"),
        ("0\\", "0\\"),
        ("[$-409h:mm", "[$-409h:mm"),
        ("[$\u{20ac}-409]0.00", "[$\u{20ac}-409]0.00"),
        ("[$$-409]0.00", "[$$-409]0.00"),
        ("[Red]0.00", "[Red]0.00"),
        ("", ""),
    ];
    for (code, expected) in cases {
        assert_eq!(without_locale_tags(code), expected, "{code}");
    }
}

#[test]
fn system_code_reads_each_system_tag_in_any_letter_case() {
    let cases = [
        ("F800", Some("dddd, mmmm d, yyyy")),
        ("f800", Some("dddd, mmmm d, yyyy")),
        ("x-sysdate", Some("dddd, mmmm d, yyyy")),
        ("X-SYSDATE", Some("dddd, mmmm d, yyyy")),
        ("F400", Some("h:mm:ss AM/PM")),
        ("f400", Some("h:mm:ss AM/PM")),
        ("x-systime", Some("h:mm:ss AM/PM")),
        ("409", None),
        ("en-US", None),
        ("", None),
    ];
    for (tag, expected) in cases {
        assert_eq!(system_code(tag), expected, "{tag}");
    }
}

#[test]
fn text_function_code_replaces_the_code_of_a_system_tag() {
    let cases = [
        ("[$-F800]dddd, mmmm dd, yyyy", "dddd, mmmm d, yyyy"),
        ("[$-x-sysdate]d", "dddd, mmmm d, yyyy"),
        ("[$-f400]h:mm", "h:mm:ss AM/PM"),
        ("[$-x-systime]h:mm", "h:mm:ss AM/PM"),
        ("0;[$-F400]0", "h:mm:ss AM/PM"),
        ("[$-409]h:mm:ss", "h:mm:ss"),
        ("\"[$-F800]\"0", "\"[$-F800]\"0"),
        ("[$\u{20ac}-409]0.00", "[$\u{20ac}-409]0.00"),
    ];
    for (code, expected) in cases {
        assert_eq!(text_function_code(code), expected, "{code}");
    }
}
