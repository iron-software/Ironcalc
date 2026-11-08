#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]

use crate::{
    formatter::{
        dates::DateSystem,
        format::{format_number, format_number_with_date_system},
    },
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("de").unwrap()
}

#[test]
fn simple_test() {
    let locale = get_default_locale();
    let b = format_number(46015.0, "m.d.yy", locale);
    assert_eq!(b.text, "12.24.25");
}

// IronXL: a date section shows the dates of ADR 029 of IronXL, as Excel shows them. Serial 1 is
// 1 January 1900, serial 60 is the fictional 29 February 1900 and shows 28 February 1900, and
// serial 0 shows 31 December 1899. The weekday is the weekday of Excel: serial 1 is a Sunday.

#[test]
fn serials_before_march_1900_show_the_dates_of_excel() {
    let locale = get_locale("en").unwrap();
    let cases = [
        (1.0, "d", "1"),
        (1.0, "d h:mm:ss", "1 0:00:00"),
        (1.0, "m/d/yyyy", "1/1/1900"),
        (1.0, "dddd", "Sunday"),
        (1.0, "ddd d mmm", "Sun 1 Jan"),
        (0.0, "dddd", "Saturday"),
        (2.0, "ddd", "Mon"),
        (59.0, "dddd", "Tuesday"),
        (60.0, "dddd", "Wednesday"),
        (61.0, "dddd", "Thursday"),
        (2.0, "m/d/yyyy", "1/2/1900"),
        (59.0, "m/d/yyyy", "2/28/1900"),
        (60.0, "m/d/yyyy", "2/28/1900"),
        (61.0, "m/d/yyyy", "3/1/1900"),
        (0.5, "d", "31"),
        (0.0, "m/d/yyyy", "12/31/1899"),
        (59.9999999, "m/d/yyyy h:mm", "2/28/1900 0:00"),
        (45000.0, "m/d/yyyy", "3/15/2023"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, expected, "{value} {format}");
    }
}

#[test]
fn the_1904_date_system_has_the_same_bounds() {
    let locale = get_locale("en").unwrap();
    let date_1904 = DateSystem::Date1904;
    let cases = [
        (-0.25, "h:mm", "#VALUE!"),
        (2_957_004.0, "h:mm", "#VALUE!"),
        (2_957_003.999_999_9, "h:mm:ss", "#VALUE!"),
        (2_957_003.0, "m/d/yyyy", "12/31/9999"),
        (0.0, "m/d/yyyy", "1/1/1904"),
        (1.0, "m/d/yyyy", "1/2/1904"),
        (59.0, "m/d/yyyy", "2/29/1904"),
        (0.0, "dddd", "Friday"),
        (59.0, "ddd", "Mon"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number_with_date_system(value, format, locale, date_1904);
        assert_eq!(formatted.text, expected, "{value} {format}");
    }
}
