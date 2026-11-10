#![allow(clippy::unwrap_used)]

// IronXL: the text section of a code shows a text value: the fourth section, or the only section
// when it holds `@`.

use crate::formatter::format::format_text;

#[test]
fn format_text_shows_the_text_section() {
    let cases = [
        ("0;0;0;\"t:\"@", "t:abc"),
        ("\"t:\"@", "t:abc"),
        ("@", "abc"),
        ("@ @", "abc abc"),
        ("0;0;0;(@)", "(abc)"),
        ("0;0;0;\\x@", "xabc"),
        ("0;0;0;.@", ".abc"),
        ("0;0;0;_(@_)", " abc "),
        ("0;0;0;*-@", " abc"),
        ("0;0;0;[Red]@", "abc"),
        ("[$\u{20ac}-409]@", "\u{20ac}abc"),
        ("0;0;0;\"none\"", "none"),
        ("0;0;0;", ""),
        (";;;", ""),
        ("0;0;0;\"\"", ""),
        ("0;0;\";\";", ""),
        ("0;0;0\\;;", ""),
    ];
    for (code, expected) in cases {
        assert_eq!(
            format_text("abc", code).as_deref(),
            Some(expected),
            "{code}"
        );
    }
}

#[test]
fn format_text_gives_none_without_a_text_section() {
    let cases = [
        "0",
        "\"t:\"",
        "0;\"t:\"@",
        "0;0;\"t:\"@",
        "0;0;0;0@",
        "0;0;0;d@",
        "0;0;0;General",
        "0;0;0;[Bad]@",
        "0;0;0;@;@",
        "0;0;",
        "0;0;\";\"",
        "0;0;0\\;",
        "",
    ];
    for code in cases {
        assert_eq!(format_text("abc", code), None, "{code}");
    }
}
