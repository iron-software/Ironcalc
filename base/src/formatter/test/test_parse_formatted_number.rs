#![allow(clippy::unwrap_used)]

use crate::{
    formatter::{
        dates::DateSystem,
        format::{
            parse_date, parse_date_time, parse_formatted_number,
            parse_formatted_number_with_date_system,
        },
    },
    locale::{get_default_locale, get_locale},
};

const PARSE_ERROR_MSG: &str = "Could not parse number";

fn parse(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_default_locale();
    parse_formatted_number(input, currencies, locale)
}

fn parse_de(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("de").unwrap();
    parse_formatted_number(input, currencies, locale)
}

fn parse_fr(input: &str, currencies: &[&str]) -> Result<(f64, Option<String>), String> {
    let locale = get_locale("fr").unwrap();
    parse_formatted_number(input, currencies, locale)
}

#[test]
fn numbers() {
    // whole numbers
    assert_eq!(parse("400", &["$"]), Ok((400.0, None)));

    // decimal numbers
    assert_eq!(parse("4.456", &["$"]), Ok((4.456, None)));

    // scientific notation
    assert_eq!(
        parse("23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.123456789e-11", &["$"]),
        Ok((2.123456789e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E-9", &["$"]),
        Ok((4.5e-9, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e+2", &["$"]),
        Ok((2300.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("4.5E9", &["$"]),
        Ok((4.5e9, Some("0.00E+00".to_string())))
    );

    // negative numbers
    assert_eq!(parse("-400", &["$"]), Ok((-400.0, None)));
    assert_eq!(parse("-4.456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse("-23e-12", &["$"]),
        Ok((-2.3e-11, Some("0.00E+00".to_string())))
    );

    // trims space
    assert_eq!(parse(" 400  ", &["$"]), Ok((400.0, None)));
}

#[test]
fn percentage() {
    // whole numbers
    assert_eq!(parse("400%", &["$"]), Ok((4.0, Some("#,##0%".to_string()))));
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Percentage in scientific notation will not be formatted as percentage
    assert_eq!(
        parse("23e-12%", &["$"]),
        Ok((23e-12 / 100.0, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3E4%", &["$"]),
        Ok((230.0, Some("0.00E+00".to_string())))
    );
}

#[test]
fn currency() {
    // whole numbers
    assert_eq!(
        parse("400$", &["$"]),
        Ok((400.0, Some("#,##0$".to_string())))
    );
    // decimal numbers
    assert_eq!(
        parse("4.456$", &["$"]),
        Ok((4.456, Some("#,##0.00$".to_string())))
    );
    // Currencies in scientific notation will not be formatted as currencies
    assert_eq!(
        parse("23e-12$", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("2.3e-12$", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("€23e-12", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );

    // switch side of currencies
    assert_eq!(
        parse("$400", &["$"]),
        Ok((400.0, Some("$#,##0".to_string())))
    );
    assert_eq!(
        parse("$4.456", &["$"]),
        Ok((4.456, Some("$#,##0.00".to_string())))
    );
    assert_eq!(
        parse("$23e-12", &["$"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("$2.3e-12", &["$"]),
        Ok((2.3e-12, Some("0.00E+00".to_string())))
    );
    assert_eq!(
        parse("23e-12€", &["€"]),
        Ok((2.3e-11, Some("0.00E+00".to_string())))
    );
}

#[test]
fn negative_currencies() {
    assert_eq!(
        parse("-400$", &["$"]),
        Ok((-400.0, Some("#,##0$".to_string())))
    );
    assert_eq!(
        parse("-$400", &["$"]),
        Ok((-400.0, Some("$#,##0".to_string())))
    );
    assert_eq!(
        parse("$-400", &["$"]),
        Ok((-400.0, Some("$#,##0".to_string())))
    );
}

#[test]
fn errors() {
    // Strings are not numbers
    assert_eq!(parse("One", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // Not partial parsing
    assert_eq!(parse("23 Hello", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("Hello 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    assert_eq!(parse("2 3", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // No space between
    assert_eq!(parse("- 23", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

#[test]
fn errors_wrong_currency() {
    assert_eq!(parse("123€", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
}

// German uses ',' as decimal separator and '.' as group separator
#[test]
fn numbers_de() {
    assert_eq!(parse_de("3,15", &["$"]), Ok((3.15, None)));
    assert_eq!(parse_de("-4,456", &["$"]), Ok((-4.456, None)));
    assert_eq!(
        parse_de("1.234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_de("1.234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_de("1.234.567", &["$"]),
        Ok((1234567.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_de() {
    assert_eq!(
        parse_de("12,5%", &["$"]),
        Ok((0.125, Some("#,##0.00%".to_string())))
    );
    assert_eq!(
        parse_de("1.234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_de() {
    // en-US formatted numbers are not valid in the German locale
    assert_eq!(
        parse_de("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_de("1.23,4", &["$"]).is_err());
}

// French uses ',' as decimal separator and
// '\u{202f}' (narrow no-break space) as group separator
#[test]
fn numbers_fr() {
    assert_eq!(parse_fr("3,15", &["$"]), Ok((3.15, None)));
    // "1,000" is one, not one thousand
    assert_eq!(parse_fr("1,000", &["$"]), Ok((1.0, None)));
    assert_eq!(
        parse_fr("1\u{202f}234", &["$"]),
        Ok((1234.0, Some("#,##0".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,56", &["$"]),
        Ok((1234.56, Some("#,##0.00".to_string())))
    );
    assert_eq!(
        parse_fr("-12\u{202f}345\u{202f}678", &["$"]),
        Ok((-12345678.0, Some("#,##0".to_string())))
    );
}

#[test]
fn percentage_and_currency_fr() {
    assert_eq!(
        parse_fr("12,5%", &["$"]),
        Ok((0.125, Some("#,##0.00%".to_string())))
    );
    assert_eq!(
        parse_fr("1\u{202f}234,50€", &["€"]),
        Ok((1234.5, Some("#,##0.00€".to_string())))
    );
}

#[test]
fn errors_fr() {
    // '.' is neither the decimal nor the group separator in French
    assert_eq!(parse_fr("1.000", &["$"]), Err(PARSE_ERROR_MSG.to_string()));
    // en-US formatted numbers are not valid in the French locale
    assert_eq!(
        parse_fr("1,234.56", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
    // group separators must come in groups of three
    assert!(parse_fr("1\u{202f}23,4", &["$"]).is_err());
}

#[test]
fn long_dates_en_us() {
    assert_eq!(
        parse("03/02/2024", &["$"]),
        Ok((45353.0, Some("mm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/02/2024", &["$"]),
        Ok((45353.0, Some("m/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("Mar/02/2024", &["$"]),
        Ok((45353.0, Some("mmm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("March/02/2024", &["$"]),
        Ok((45353.0, Some("mmmm/dd/yyyy".to_string())))
    );
    assert_eq!(
        parse("3/2/24", &["$"]),
        Ok((45353.0, Some("m/d/yy".to_string())))
    );

    assert_eq!(
        parse("02-10-1975", &["$"]),
        Ok((27435.0, Some("mm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-1975", &["$"]),
        Ok((27435.0, Some("m-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("Feb-10-1975", &["$"]),
        Ok((27435.0, Some("mmm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("February-10-1975", &["$"]),
        Ok((27435.0, Some("mmmm-dd-yyyy".to_string())))
    );
    assert_eq!(
        parse("2-10-75", &["$"]),
        Ok((27435.0, Some("m-dd-yy".to_string())))
    );
}

#[test]
fn iso_dates() {
    assert_eq!(
        parse("2024/03/02", &["$"]),
        Ok((45353.0, Some("yyyy/mm/dd".to_string())))
    );
    assert_eq!(
        parse("2024/March/02", &["$"]),
        Err(PARSE_ERROR_MSG.to_string())
    );
}

#[test]
fn long_dates_with_dots() {
    assert_eq!(
        parse("03.02.2024", &["$"]),
        Ok((45353.0, Some("mm.dd.yyyy".to_string())))
    );
}

// IronXL: a time text reads as the fraction of a day, with a time format.
#[test]
fn time() {
    assert_eq!(parse("12:00", &["$"]), Ok((0.5, Some("h:mm".to_string()))));
    assert_eq!(
        parse(" 12:00:00 ", &["$"]),
        Ok((0.5, Some("h:mm:ss".to_string())))
    );
    assert_eq!(
        parse("12:00:00.25", &["$"]),
        Ok((43_200.25 / 86_400.0, Some("h:mm:ss.00".to_string())))
    );
    assert_eq!(
        parse("9:05:07.125", &["$"]),
        Ok((0.3785546875, Some("h:mm:ss.000".to_string())))
    );
    assert_eq!(
        parse("6:30:15.5 PM", &["$"]),
        Ok((66_615.5 / 86_400.0, Some("h:mm:ss.0 AM/PM".to_string())))
    );
    assert_eq!(
        parse("12:30am", &["$"]),
        Ok((30.0 / 1_440.0, Some("h:mm AM/PM".to_string())))
    );
    assert_eq!(
        parse("12:30 PM", &["$"]),
        Ok((750.0 / 1_440.0, Some("h:mm AM/PM".to_string())))
    );
    for text in [
        "12:00:00.2.5",
        "12:00:00.x",
        "12:00:00.",
        "12:00:00.1234",
        "12:00.5",
        "24:00",
        "12:60",
        "12:00:60",
        "123:00",
        "1:2:3:4",
        ":30",
        "12:",
        "13:00 PM",
        "0:30 AM",
        "12:00 XM",
        "AM",
    ] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}

// IronXL: a date text, then white space, then a time text, reads as the date serial number
// plus the fraction of the day, with the date format and the time format.
#[test]
fn date_and_time() {
    assert_eq!(
        parse("1/2/2020 12:00", &["$"]),
        Ok((43832.5, Some("m/d/yyyy h:mm".to_string())))
    );
    assert_eq!(
        parse(" 1/2/2020   12:00 ", &["$"]),
        Ok((43832.5, Some("m/d/yyyy h:mm".to_string())))
    );
    assert_eq!(
        parse("1/2/2020 12:00:00.25", &["$"]),
        Ok((
            43832.0 + 43_200.25 / 86_400.0,
            Some("m/d/yyyy h:mm:ss.00".to_string())
        ))
    );
    assert_eq!(
        parse("1/2/2020 6:30 PM", &["$"]),
        Ok((
            43832.0 + 1_110.0 / 1_440.0,
            Some("m/d/yyyy h:mm AM/PM".to_string())
        ))
    );
    assert_eq!(
        parse("2020-01-02 12:00", &["$"]),
        Ok((43832.5, Some("yyyy-mm-dd h:mm".to_string())))
    );
    assert_eq!(
        parse("02.01.2020\t6:00", &["$"]),
        Ok((43862.25, Some("mm.dd.yyyy h:mm".to_string())))
    );
    assert_eq!(
        parse_de("2.1.2020 12:00", &["€"]),
        Ok((43832.5, Some("d.m.yyyy h:mm".to_string())))
    );
    for text in [
        "1/2/2020 12:60",
        "1/2/2020 abc",
        "1/2/2020 24:00",
        "2/30/2020 12:00",
        "abc 12:00",
        "12:00 1/2/2020",
        "1/2/2020 12:00 1",
        "1/2 12:00",
        "1/2/2020T12:00",
    ] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}

// IronXL: the date part of a date and time text gives a serial number of the date system.
#[test]
fn date_and_time_in_the_1904_date_system() {
    let locale = get_default_locale();
    assert_eq!(
        parse_formatted_number_with_date_system(
            "1/2/2020 12:00",
            &["$"],
            locale,
            DateSystem::Date1904
        ),
        Ok((42370.5, Some("m/d/yyyy h:mm".to_string())))
    );
    assert_eq!(
        parse_date_time("1/2/2020", locale, DateSystem::Date1900),
        Err("Not a valid date and time".to_string())
    );
}

fn ok(value: f64, format: &str) -> Result<(f64, Option<String>), String> {
    Ok((value, Some(format.to_string())))
}

// IronXL: a date text can have white space before and after it, as in Excel.
#[test]
fn date_text_with_white_space() {
    assert_eq!(parse(" 1/2/2020", &["$"]), ok(43832.0, "m/d/yyyy"));
    assert_eq!(parse("1/2/2020 ", &["$"]), ok(43832.0, "m/d/yyyy"));
    assert_eq!(parse(" 1/2/2020 ", &["$"]), ok(43832.0, "m/d/yyyy"));
    assert_eq!(parse("\t1/2/2020\n", &["$"]), ok(43832.0, "m/d/yyyy"));
    assert_eq!(parse(" 2020-01-02 ", &["$"]), ok(43832.0, "yyyy-mm-dd"));
    assert_eq!(parse(" 2-Jan-2020 ", &["$"]), ok(43832.0, "d-mmm-yyyy"));
    assert_eq!(
        parse_date(" 1/2/2020 ", get_default_locale(), DateSystem::Date1900),
        Ok((43832, "m/d/yyyy".to_string()))
    );
}

// IronXL: a month name in the first or the second part is the month, in any letter case.
#[test]
fn date_text_with_a_month_name() {
    assert_eq!(parse("2-Jan-2020", &["$"]), ok(43832.0, "d-mmm-yyyy"));
    assert_eq!(parse("2-jan-2020", &["$"]), ok(43832.0, "d-mmm-yyyy"));
    assert_eq!(parse("2-JAN-2020", &["$"]), ok(43832.0, "d-mmm-yyyy"));
    assert_eq!(parse("2-January-2020", &["$"]), ok(43832.0, "d-mmmm-yyyy"));
    assert_eq!(
        parse("02-january-2020", &["$"]),
        ok(43832.0, "dd-mmmm-yyyy")
    );
    assert_eq!(parse("2-Jan-20", &["$"]), ok(43832.0, "d-mmm-yy"));
    assert_eq!(parse("2/Jan/2020", &["$"]), ok(43832.0, "d/mmm/yyyy"));
    assert_eq!(parse("2.Jan.2020", &["$"]), ok(43832.0, "d.mmm.yyyy"));
    assert_eq!(parse("Jan-2-2020", &["$"]), ok(43832.0, "mmm-d-yyyy"));
    assert_eq!(parse("JAN-2-2020", &["$"]), ok(43832.0, "mmm-d-yyyy"));
    // A full name of 4 letters is not the year of an ISO date.
    assert_eq!(parse("June-2-2020", &["$"]), ok(43984.0, "mmmm-d-yyyy"));
    // Number parts keep the order of the locale; a month name sets its own position.
    let es = get_locale("es").unwrap();
    assert_eq!(
        parse_formatted_number("ENE-15-2023", &[], es),
        ok(44941.0, "mmm-dd-yyyy")
    );
    assert_eq!(
        parse_formatted_number("15-Ene-2023", &[], es),
        ok(44941.0, "dd-mmm-yyyy")
    );
}

// IronXL: a long date is a month name, a day, an optional comma, and a year, or a day, a month
// name, and a year, with white space between them.
#[test]
fn long_date_text() {
    assert_eq!(parse("Jan 2, 2020", &["$"]), ok(43832.0, "mmm d, yyyy"));
    assert_eq!(
        parse("January 2, 2020", &["$"]),
        ok(43832.0, "mmmm d, yyyy")
    );
    assert_eq!(parse("Jan 2 2020", &["$"]), ok(43832.0, "mmm d yyyy"));
    assert_eq!(parse("Jan 2,2020", &["$"]), ok(43832.0, "mmm d, yyyy"));
    assert_eq!(
        parse(" jan  02,  2020 ", &["$"]),
        ok(43832.0, "mmm dd, yyyy")
    );
    assert_eq!(parse("Jan 2, 20", &["$"]), ok(43832.0, "mmm d, yy"));
    assert_eq!(parse("2 Jan 2020", &["$"]), ok(43832.0, "d mmm yyyy"));
    assert_eq!(parse("2 January 2020", &["$"]), ok(43832.0, "d mmmm yyyy"));
    assert_eq!(parse("2\tJAN\t20", &["$"]), ok(43832.0, "d mmm yy"));
    assert_eq!(parse("Feb 29, 2020", &["$"]), ok(43890.0, "mmm dd, yyyy"));
}

// IronXL: a date text with a month name, then white space, then a time text.
#[test]
fn date_text_with_a_month_name_and_a_time() {
    assert_eq!(
        parse("2-Jan-2020 12:00", &["$"]),
        ok(43832.5, "d-mmm-yyyy h:mm")
    );
    assert_eq!(
        parse("Jan 2, 2020 12:00", &["$"]),
        ok(43832.5, "mmm d, yyyy h:mm")
    );
    assert_eq!(
        parse(" 2 Jan 2020 6:30 PM ", &["$"]),
        ok(43832.0 + 1_110.0 / 1_440.0, "d mmm yyyy h:mm AM/PM")
    );
    assert_eq!(
        parse("Jan 2 2020  6:30   pm", &["$"]),
        ok(43832.0 + 1_110.0 / 1_440.0, "mmm d yyyy h:mm AM/PM")
    );
    assert_eq!(
        parse("January 2, 2020 12:00:00.5", &["$"]),
        ok(43832.5 + 0.5 / 86_400.0, "mmmm d, yyyy h:mm:ss.0")
    );
    assert_eq!(
        parse_date_time("Jan 2, 2020", get_default_locale(), DateSystem::Date1900),
        Err("Not a valid date and time".to_string())
    );
}

// IronXL: the new date texts give a serial number of the 1904 date system.
#[test]
fn date_text_with_a_month_name_in_the_1904_date_system() {
    let locale = get_default_locale();
    for text in ["2-Jan-2020", "Jan 2, 2020", " 1/2/2020 ", "2 January 2020"] {
        assert_eq!(
            parse_formatted_number_with_date_system(text, &["$"], locale, DateSystem::Date1904)
                .map(|(value, _)| value),
            Ok(42370.0),
            "{text}"
        );
    }
    assert_eq!(
        parse_formatted_number_with_date_system(
            "Jan 2, 2020 12:00",
            &["$"],
            locale,
            DateSystem::Date1904
        ),
        ok(42370.5, "mmm d, yyyy h:mm")
    );
    // The 1904 date system has no date before 1 January 1904.
    assert!(parse_formatted_number_with_date_system(
        "Dec 31, 1903",
        &["$"],
        locale,
        DateSystem::Date1904
    )
    .is_err());
}

// IronXL: an invalid date text with a month name does not read as a number.
#[test]
fn date_text_with_a_month_name_errors() {
    for text in [
        "32-Jan-2020",
        "2-Foo-2020",
        "Jan-Feb-2020",
        "2-Jan-2020x",
        "2-Jan-202",
        "2020-Jan-02",
        "Jan 32, 2020",
        "Feb 30, 2020",
        "Jan 2, 2020 12:60",
        "Jan 2, 2020 12:00 PM 1",
        "Jan 2,, 2020",
        "Jan 2 , 2020",
        "Jan, 2 2020",
        "Jan 2,",
        "Jan 2",
        "Jan",
        "2 Jan",
        "2 Jan, 2020",
        "Foo 2, 2020",
        "2 Foo 2020",
        "1 2 2020",
        "Jan Jan 2020",
        "Jan 2, 2020x",
    ] {
        assert!(parse(text, &["$"]).is_err(), "{text}");
    }
}
