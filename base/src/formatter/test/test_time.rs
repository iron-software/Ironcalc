#![allow(clippy::unwrap_used)]

use crate::{
    formatter::format::format_number,
    locale::{get_locale, Locale},
};

fn get_default_locale() -> &'static Locale {
    get_locale("en").unwrap()
}

#[test]
fn simple_test() {
    let locale = get_default_locale();
    let format = "h:mm AM/PM";
    let value = 16.001_423_611_111_11; // =1/86400 => 12:02 AM
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "12:02 AM");
}

#[test]
fn padded_vs_unpadded() {
    let locale = get_default_locale();
    let padded_format = "hh:mm:ss AM/PM";
    let unpadded_format = "h:m:s AM/PM";
    let value = 0.25351851851851853; // => 6:05:04 AM (21904/(24*60*60)) where 21904 = 6 * 3600 + 5*60 + 4
    let formatted = format_number(value, padded_format, locale);
    assert_eq!(formatted.text, "06:05:04 AM");

    let formatted = format_number(value, unpadded_format, locale);
    assert_eq!(formatted.text, "6:5:4 AM");
}

#[test]
fn elapsed_hour() {
    let locale = get_default_locale();
    let format = "[hh]:mm:ss";
    // 1.5 days = 36 hours
    let value = 1.5;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "36:00:00");
}

#[test]
fn elapsed_minute() {
    let locale = get_default_locale();
    let format = "[mm]:ss";
    // 2 days = 2880 minutes
    let value = 2.0;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "2880:00");
}

#[test]
fn elapsed_second() {
    let locale = get_default_locale();
    let format = "[ss]";
    // 0.5 days = 43200 seconds
    let value = 0.5;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "43200");
}

#[test]
fn elapsed_hour_padded() {
    let locale = get_default_locale();
    let format = "[hh]:mm:ss";
    // 0.1 days = 2.4 hours
    let value = 0.1;
    let formatted = format_number(value, format, locale);
    assert_eq!(formatted.text, "02:24:00");
}

// IronXL: a section with a time part rounds the value to whole seconds once, with a carry to
// the minute, the hour, and the day. An elapsed code counts in i64 and does not clamp.

#[test]
fn elapsed_codes_of_a_large_value_round_to_a_whole_second() {
    let locale = get_default_locale();
    let value = 45000.545185185;
    assert_eq!(format_number(value, "[ss]", locale).text, "3888047104");
    assert_eq!(format_number(value, "[s]", locale).text, "3888047104");
    assert_eq!(format_number(value, "[h]", locale).text, "1080013");
    assert_eq!(format_number(value, "[hh]", locale).text, "1080013");
    assert_eq!(format_number(value, "[mm]", locale).text, "64800785");
    assert_eq!(format_number(value, "[m]", locale).text, "64800785");
    assert_eq!(format_number(1234.5, "[ss]", locale).text, "106660800");
}

#[test]
fn elapsed_codes_of_the_last_excel_date_do_not_clamp() {
    let locale = get_default_locale();
    let value = 2958465.9999884;
    assert_eq!(format_number(value, "[s]", locale).text, "255611462399");
    assert_eq!(format_number(value, "[ss]", locale).text, "255611462399");
    assert_eq!(format_number(value, "[mm]", locale).text, "4260191039");
    assert_eq!(format_number(value, "[m]", locale).text, "4260191039");
    assert_eq!(format_number(value, "[h]", locale).text, "71003183");
    assert_eq!(format_number(value, "[hh]", locale).text, "71003183");
}

#[test]
fn a_time_section_carries_a_rounded_second_to_the_minute_the_hour_and_the_day() {
    let locale = get_default_locale();
    let value = 0.9999999;
    assert_eq!(format_number(value, "[h]:mm:ss", locale).text, "24:00:00");
    assert_eq!(format_number(value, "h:mm:ss", locale).text, "0:00:00");
    assert_eq!(
        format_number(value, "hh:mm:ss AM/PM", locale).text,
        "12:00:00 AM"
    );
    assert_eq!(format_number(value, "[s]", locale).text, "86400");
    assert_eq!(format_number(value, "[m]", locale).text, "1440");
    assert_eq!(format_number(value, "[mm]:ss", locale).text, "1440:00");
    assert_eq!(
        format_number(45000.9999999, "d h:mm:ss", locale).text,
        "16 0:00:00"
    );
    assert_eq!(
        format_number(45000.9999999, "yyyy-mm-dd hh:mm:ss", locale).text,
        "2023-03-16 00:00:00"
    );
    // 12:29:59.6 rounds to 12:30:00
    let value = (12.0 * 3600.0 + 29.0 * 60.0 + 59.6) / 86400.0;
    assert_eq!(format_number(value, "h:mm:ss", locale).text, "12:30:00");
    assert_eq!(format_number(value, "h:mm", locale).text, "12:30");
}

#[test]
fn larger_units_and_a_section_without_a_time_part_do_not_round() {
    let locale = get_default_locale();
    assert_eq!(format_number(0.52, "h:mm", locale).text, "12:28");
    // 12:29:40 shows 12:29 with h:mm
    let value = (12.0 * 3600.0 + 29.0 * 60.0 + 40.0) / 86400.0;
    assert_eq!(format_number(value, "h:mm", locale).text, "12:29");
    assert_eq!(format_number(value, "[h]", locale).text, "12");
    assert_eq!(format_number(45000.9999999, "d", locale).text, "15");
    assert_eq!(
        format_number(45000.9999999, "yyyy-mm-dd", locale).text,
        "2023-03-15"
    );
}

#[test]
fn every_time_code_reads_the_same_rounded_second() {
    let locale = get_default_locale();
    // 23:59:59.5 rounds to the next day
    let value = 45000.0 + 86399.5 / 86400.0;
    assert_eq!(format_number(value, "d h", locale).text, "16 0");
    assert_eq!(format_number(value, "d h AM/PM", locale).text, "16 12 AM");
    assert_eq!(format_number(value, "d h A/P", locale).text, "16 12 A");
    assert_eq!(format_number(value, "d hh a/p", locale).text, "16 12 a");
    assert_eq!(format_number(value, "d m:ss", locale).text, "16 0:00");
    assert_eq!(format_number(value, "d [hh]", locale).text, "16 1080024");
}

// IronXL: `.` and 1 to 3 `0` right after s, ss, [s], or [ss] are the digits after the seconds.
// The section rounds the value once to these digits, with a carry to the larger units.

#[test]
fn digits_after_the_seconds_show_the_fraction_of_a_second() {
    let locale = get_default_locale();
    let cases = [
        (0.5000028935185185, "h:mm:ss.000", "12:00:00.250"),
        (0.5000028935185185, "H:MM:SS.000", "12:00:00.250"),
        (45000.545185185, "[ss].00", "3888047104.00"),
        (45000.545185185, "[s].0", "3888047104.0"),
        (0.5 + 1.25 / 86400.0, "mm:ss.0", "00:01.3"),
        (0.5 + 1.5 / 86400.0, "[ss].00", "43201.50"),
        (0.5 + 1.5 / 86400.0, "s.0", "1.5"),
        (0.5 + 1.5 / 86400.0, "[SS].00", "43201.50"),
        (0.99999, "[h]:mm:ss.0", "23:59:59.1"),
        (0.5 + 1.5 / 86400.0, "h:mm:ss.0 AM/PM", "12:00:01.5 PM"),
        (0.5 + 1.5 / 86400.0, "ss.0 \"s\"", "01.5 s"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, expected, "{format}");
        assert_eq!(formatted.error, None, "{format}");
    }
}

#[test]
fn digits_after_the_seconds_round_once_with_a_carry() {
    let locale = get_default_locale();
    let cases = [
        (0.9999999999, "h:mm:ss.00", "0:00:00.00"),
        (0.9999999999, "[h]:mm:ss.00", "24:00:00.00"),
        (45000.9999999999, "d h:mm:ss.000", "16 0:00:00.000"),
        (0.5 + 59.96 / 86400.0, "mm:ss.0", "01:00.0"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, expected, "{format}");
    }
}

#[test]
fn a_section_with_more_digit_parts_rounds_to_the_largest_count() {
    let locale = get_default_locale();
    let value = 0.5 + 1.256 / 86400.0;
    let formatted = format_number(value, "ss.0 ss.000", locale);
    assert_eq!(formatted.text, "01.2 01.256");
}

#[test]
fn other_zeros_in_a_time_section_stay_invalid() {
    let locale = get_default_locale();
    for format in [
        "h:mm:ss.0000",
        "h:mm:ss0",
        "h:mm.0",
        "h:mm:ss 0",
        "h:mm:ss.000.0",
    ] {
        let formatted = format_number(0.5, format, locale);
        assert_eq!(formatted.text, "#VALUE!", "{format}");
    }
}

#[test]
fn codes_without_digits_after_the_seconds_keep_their_text() {
    let locale = get_default_locale();
    let cases = [
        (0.5, "h:mm:ss", "12:00:00"),
        (0.5, "h:mm:ss.", "12:00:00."),
        (0.5, "h:mm:ss.\"x\"", "12:00:00.x"),
        (1.5, "0.00", "1.50"),
        (0.5, "h:mm:ss.000;0.00", "12:00:00.000"),
        (-1.5, "h:mm:ss.000;0.00", "1.50"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, expected, "{format}");
    }
}

// IronXL: a date or time section shows a serial from 0 to 31 December 9999. A negative serial,
// and a serial after 31 December 9999 also after the rounding, give #VALUE!, as Excel gives.

#[test]
fn a_negative_serial_in_a_date_or_time_section_gives_an_error() {
    let locale = get_default_locale();
    for format in ["h:mm", "[h]:mm", "d", "[s]", "h:mm:ss.0"] {
        let formatted = format_number(-0.25, format, locale);
        assert_eq!(formatted.text, "#VALUE!", "{format}");
        assert!(formatted.error.is_some(), "{format}");
    }
}

#[test]
fn a_negative_section_shows_the_absolute_value() {
    let locale = get_default_locale();
    assert_eq!(format_number(-0.25, "h:mm;h:mm", locale).text, "6:00");
    assert_eq!(format_number(-0.25, "h:mm;[h]:mm;0", locale).text, "6:00");
    assert_eq!(format_number(0.0, "h:mm;h:mm", locale).text, "0:00");
}

#[test]
fn a_serial_after_the_last_date_gives_an_error() {
    let locale = get_default_locale();
    let cases = [
        (110_000_000_000_000.0, "h:mm"),
        (110_000_000_000_000.0, "d"),
        (2_958_466.0, "h:mm"),
        (2_958_466.0, "[s]"),
        (2_958_466.0, "m/d/yyyy"),
        (2_958_465.999_999_9, "[s]"),
        (2_958_465.999_999_9, "[s].0"),
        (2_958_465.999_999_9, "h:mm:ss"),
    ];
    for (value, format) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, "#VALUE!", "{value} {format}");
    }
}

#[test]
fn the_last_date_keeps_its_text() {
    let locale = get_default_locale();
    let cases = [
        (2_958_465.999_999_9, "d", "31"),
        (2_958_465.999_99, "[s].0", "255611462399.1"),
        (2_958_465.0, "m/d/yyyy", "12/31/9999"),
        (2_958_465.5, "m/d/yyyy h:mm", "12/31/9999 12:00"),
        (0.0, "h:mm", "0:00"),
    ];
    for (value, format, expected) in cases {
        let formatted = format_number(value, format, locale);
        assert_eq!(formatted.text, expected, "{value} {format}");
    }
}
