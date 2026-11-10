use chrono::Datelike;

use crate::{locale::Locale, number_format::to_precision};

use super::{
    dates::{date_of_serial, date_to_serial_number, weekday_of_serial, DateSystem},
    parser::{ParsePart, Parser, TextToken},
};

const SECONDS_PER_DAY: i64 = 86_400;

/// IronXL: the serial `value` as a count of units, rounded to the nearest unit. A day has
/// `units_per_day` units.
fn whole_units(value: f64, units_per_day: i64) -> i64 {
    (value * units_per_day as f64).round() as i64
}

/// IronXL: true when a date or time section can show the serial `value` with the rounded `day`:
/// the value is not negative, and the value and the day are not after 31 December 9999.
fn is_shown_serial(value: f64, day: i64, date_system: DateSystem) -> bool {
    let end = date_system.maximum_serial() as i64 + 1;
    (0.0..end as f64).contains(&value) && day < end
}

/// IronXL: true for a token that shows a part of the time of day or an elapsed time.
fn is_time_token(token: &TextToken) -> bool {
    matches!(
        token,
        TextToken::Hour
            | TextToken::HourPadded
            | TextToken::Minute
            | TextToken::MinutePadded
            | TextToken::Second
            | TextToken::SecondPadded
            | TextToken::ElapsedHour
            | TextToken::ElapsedHourPadded
            | TextToken::ElapsedMinute
            | TextToken::ElapsedMinutePadded
            | TextToken::ElapsedSecond
            | TextToken::ElapsedSecondPadded
            | TextToken::AMPM
            | TextToken::AP(_)
    )
}

pub struct Formatted {
    pub color: Option<i32>,
    pub text: String,
    pub error: Option<String>,
}

/// Returns the vector of chars of the fractional part of a *positive* number:
/// 3.1415926 ==> ['1', '4', '1', '5', '9', '2', '6']
fn get_fract_part(value: f64, precision: i32, int_len: usize) -> Vec<char> {
    let b = format!("{:.1$}", value.fract(), precision as usize)
        .chars()
        .collect::<Vec<char>>();
    let l = b.len() - 1;
    let mut last_non_zero = b.len() - 1;
    for i in 0..l {
        if b[l - i] != '0' {
            last_non_zero = l - i + 1;
            break;
        }
    }
    if last_non_zero < 2 {
        return vec![];
    }
    let max_len = if int_len > 15 {
        2_usize
    } else {
        15_usize - int_len + 1
    };
    let last_non_zero = usize::min(last_non_zero, max_len + 1);
    b[2..last_non_zero].to_vec()
}

/// Return true if we need to add a separator in position digit_index
/// It normally happens if if digit_index -1 is 3, 6, 9,... digit_index ≡ 1 mod 3
fn use_group_separator(use_thousands: bool, digit_index: i32, group_sizes: &str) -> bool {
    if use_thousands {
        if group_sizes == "#,##0.###" {
            if digit_index > 1 && (digit_index - 1) % 3 == 0 {
                return true;
            }
        } else if group_sizes == "#,##,##0.###"
            && (digit_index == 3 || (digit_index > 3 && digit_index % 2 == 0))
        {
            return true;
        }
    }
    false
}

/// IronXL: the en-US code of the long date of the system, for the tag `[$-F800]`.
const SYSTEM_LONG_DATE: &str = "dddd, mmmm d, yyyy";
/// IronXL: the en-US code of the long time of the system, for the tag `[$-F400]`.
const SYSTEM_LONG_TIME: &str = "h:mm:ss AM/PM";

/// IronXL: `code` without each locale tag without a currency symbol, such as `[$-409]`, and the
/// text of each tag after `[$-`. A tag in quoted text or after `\` stays. A tag before a `.`
/// becomes the empty text `""`, so `ss[$-409].0` becomes `ss"".0`, which the parser refuses, as
/// Excel does.
fn split_locale_tags(code: &str) -> (String, Vec<&str>) {
    let mut kept = String::with_capacity(code.len());
    let mut tags = Vec::new();
    let mut rest = code;
    let mut quoted = false;
    while let Some(c) = rest.chars().next() {
        if !quoted && rest.starts_with("[$-") {
            if let Some(end) = rest.find(']') {
                tags.push(&rest[3..end]);
                rest = &rest[end + 1..];
                if rest.starts_with('.') {
                    kept.push_str(r#""""#);
                }
                continue;
            }
        }
        let width = match c {
            // An escaped character is one unit with its `\`.
            '\\' if !quoted => 1 + rest[1..].chars().next().map_or(0, char::len_utf8),
            _ => c.len_utf8(),
        };
        kept.push_str(&rest[..width]);
        quoted ^= c == '"';
        rest = &rest[width..];
    }
    (kept, tags)
}

/// IronXL: a copy of the number format `code` for the parser: each locale tag without a currency
/// symbol goes, as `split_locale_tags` tells. The parser reads a currency symbol after `[$` and
/// refuses a tag without one, but Excel accepts it. The engine does not use the locale of a tag.
pub fn without_locale_tags(code: &str) -> String {
    split_locale_tags(code).0
}

/// IronXL: the en-US code of a system tag, the text after `[$-` in any letter case: the long date
/// for `F800` and `x-sysdate`, the long time for `F400` and `x-systime`. None for another tag.
pub fn system_code(tag: &str) -> Option<&'static str> {
    if tag.eq_ignore_ascii_case("f800") || tag.eq_ignore_ascii_case("x-sysdate") {
        Some(SYSTEM_LONG_DATE)
    } else if tag.eq_ignore_ascii_case("f400") || tag.eq_ignore_ascii_case("x-systime") {
        Some(SYSTEM_LONG_TIME)
    } else {
        None
    }
}

/// IronXL: the code that `TEXT` formats with. A system tag in any section replaces the whole code
/// with its en-US code, as Excel shows it in the en-US locale. Else the locale tags go, as
/// `without_locale_tags` tells.
pub fn text_function_code(code: &str) -> String {
    let (kept, tags) = split_locale_tags(code);
    match tags.into_iter().find_map(system_code) {
        Some(system) => system.to_string(),
        None => kept,
    }
}

/// IronXL: the text that the code `format` shows for the text `value`, as Excel shows it. The text
/// section is the fourth section, or the only section when it holds `@`. Quoted text and literals
/// show as they are, `@` shows `value`, and `_x` and `*x` show one space, as for a number. A color
/// shows nothing. None when the code has no text section, or the section holds a token for a
/// number or a date, such as `0`.
pub fn format_text(value: &str, format: &str) -> Option<String> {
    let mut parser = Parser::new(format);
    parser.parse();
    let part = match parser.parts.as_slice() {
        [part] => part,
        [_, _, _, part] => part,
        _ => return None,
    };
    let ParsePart::Number(p) = part else {
        return None;
    };
    if parser.parts.len() == 1 && !p.tokens.iter().any(|t| matches!(t, TextToken::Raw)) {
        return None;
    }
    let mut text = p.currency.map(String::from).unwrap_or_default();
    for token in &p.tokens {
        match token {
            TextToken::Literal(c) => text.push(*c),
            TextToken::Text(t) => text.push_str(t),
            TextToken::Ghost(_) | TextToken::Spacer(_) => text.push(' '),
            TextToken::Raw => text.push_str(value),
            _ => return None,
        }
    }
    Some(text)
}

pub fn format_number(value_original: f64, format: &str, locale: &Locale) -> Formatted {
    format_number_with_date_system(value_original, format, locale, DateSystem::Date1900)
}

/// IronXL: formats a number like `format_number`. A date format reads the value as a serial
/// number of `date_system`.
pub fn format_number_with_date_system(
    value_original: f64,
    format: &str,
    locale: &Locale,
    date_system: DateSystem,
) -> Formatted {
    let mut parser = Parser::new(format);
    parser.parse();
    let parts = parser.parts;
    // There are four parts:
    // 1) Positive numbers
    // 2) Negative numbers
    // 3) Zero
    // 4) Text
    // If you specify only one section of format code, the code in that section is used for all numbers.
    // If you specify two sections of format code, the first section of code is used
    // for positive numbers and zeros, and the second section of code is used for negative numbers.
    // When you skip code sections in your number format,
    // you must include a semicolon for each of the missing sections of code.
    // You can use the ampersand (&) text operator to join, or concatenate, two values.
    let mut value = value_original;
    let part;
    match parts.len() {
        1 => {
            part = &parts[0];
        }
        2 => {
            if value >= 0.0 {
                part = &parts[0]
            } else {
                value = -value;
                part = &parts[1];
            }
        }
        3 => {
            if value > 0.0 {
                part = &parts[0]
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        4 => {
            if value > 0.0 {
                part = &parts[0]
            } else if value < 0.0 {
                value = -value;
                part = &parts[1];
            } else {
                value = 0.0;
                part = &parts[2];
            }
        }
        _ => {
            return Formatted {
                text: "#VALUE!".to_owned(),
                color: None,
                error: Some("Too many parts".to_owned()),
            };
        }
    }
    match part {
        ParsePart::Error(..) => Formatted {
            text: "#VALUE!".to_owned(),
            color: None,
            error: Some("Problem parsing format string".to_owned()),
        },
        ParsePart::General(..) => {
            // FIXME: This is "General formatting"
            // We should have different codepaths for general formatting and errors
            let value_abs = value.abs();
            if (1.0e-8..1.0e+11).contains(&value_abs) {
                let mut text = format!("{value:.9}");
                text = text.trim_end_matches('0').trim_end_matches('.').to_string();
                if locale.numbers.symbols.decimal != "." {
                    text = text.replace('.', &locale.numbers.symbols.decimal.to_string());
                }
                Formatted {
                    text,
                    color: None,
                    error: None,
                }
            } else {
                if value_abs == 0.0 {
                    return Formatted {
                        text: "0".to_string(),
                        color: None,
                        error: None,
                    };
                }
                let exponent = value_abs.log10().floor();
                value /= 10.0_f64.powf(exponent);
                let sign = if exponent < 0.0 { '-' } else { '+' };
                let s = format!("{value:.5}");
                let mut text = format!(
                    "{}E{}{:02}",
                    s.trim_end_matches('0').trim_end_matches('.'),
                    sign,
                    exponent.abs()
                );
                if locale.numbers.symbols.decimal != "." {
                    text = text.replace('.', &locale.numbers.symbols.decimal.to_string());
                }
                Formatted {
                    text,
                    color: None,
                    error: None,
                }
            }
        }
        ParsePart::Date(p) => {
            let tokens = &p.tokens;
            let mut text = "".to_string();
            // IronXL: a section with a time part rounds the value once to the smallest unit
            // that it shows: a second, or the digits after the seconds (the largest count).
            // Each part reads this count, so a carry goes to the second, the minute, the hour,
            // and the day, and `:60` does not occur. An elapsed code counts in i64, so it does
            // not clamp.
            let decimals = tokens
                .iter()
                .map(|token| match token {
                    TextToken::SecondDecimals(count) => *count,
                    _ => 0,
                })
                .max()
                .unwrap_or(0);
            let units_per_second = 10_i64.pow(decimals as u32);
            let units_per_day = SECONDS_PER_DAY * units_per_second;
            let (day, units) = if tokens.iter().any(is_time_token) {
                let units = whole_units(value, units_per_day);
                (units.div_euclid(units_per_day), units)
            } else {
                (value as i64, 0)
            };
            // IronXL: a negative serial, and a serial after 31 December 9999 also after the
            // rounding, give #VALUE!, as Excel gives. The display text shows the General text.
            if !is_shown_serial(value, day, date_system) {
                return Formatted {
                    text: "#VALUE!".to_owned(),
                    color: None,
                    error: Some(format!("Invalid date value: '{value}'")),
                };
            }
            let elapsed_seconds = units.div_euclid(units_per_second);
            let second_of_day = elapsed_seconds.rem_euclid(SECONDS_PER_DAY);
            let hours = second_of_day / 3600;
            let minutes = second_of_day / 60 % 60;
            let seconds = second_of_day % 60;
            let date = date_of_serial(day, date_system);
            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Digit(_) => {}
                    TextToken::Period => {}
                    TextToken::Day => match date {
                        Some(date) => {
                            let day = date.day() as usize;
                            text = format!("{text}{day}");
                        }
                        None => {
                            return Formatted {
                                text: "#VALUE!".to_owned(),
                                color: None,
                                error: Some(format!("Invalid date value: '{value}'")),
                            }
                        }
                    },
                    TextToken::DayPadded => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let day = date.day() as usize;
                        text = format!("{text}{day:02}");
                    }
                    TextToken::DayNameShort => {
                        if date.is_none() {
                            return Formatted {
                                text: "#VALUE!".to_owned(),
                                color: None,
                                error: Some(format!("Invalid date value: '{value}'")),
                            };
                        }
                        let day =
                            weekday_of_serial(day, date_system).num_days_from_sunday() as usize;
                        text = format!("{}{}", text, locale.dates.day_names_short[day]);
                    }
                    TextToken::DayName => {
                        if date.is_none() {
                            return Formatted {
                                text: "#VALUE!".to_owned(),
                                color: None,
                                error: Some(format!("Invalid date value: '{value}'")),
                            };
                        }
                        let day =
                            weekday_of_serial(day, date_system).num_days_from_sunday() as usize;
                        text = format!("{}{}", text, locale.dates.day_names[day]);
                    }
                    TextToken::Month => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{text}{month}");
                    }
                    TextToken::MonthPadded => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{text}{month:02}");
                    }
                    TextToken::MonthNameShort => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{}{}", text, locale.dates.months_short[month - 1]);
                    }
                    TextToken::MonthName => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        text = format!("{}{}", text, locale.dates.months[month - 1]);
                    }
                    TextToken::MonthLetter => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        let month = date.month() as usize;
                        let months_letter = &locale.dates.months_letter[month - 1];
                        text = format!("{text}{months_letter}");
                    }
                    TextToken::YearShort => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        text = format!("{}{}", text, date.format("%y"));
                    }
                    TextToken::Year => {
                        let date = match date {
                            Some(d) => d,
                            None => {
                                return Formatted {
                                    text: "#VALUE!".to_owned(),
                                    color: None,
                                    error: Some(format!("Invalid date value: '{value}'")),
                                }
                            }
                        };
                        text = format!("{}{}", text, date.year());
                    }
                    TextToken::Hour => {
                        let mut hour = hours;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour}");
                    }
                    TextToken::HourPadded => {
                        let mut hour = hours;
                        if p.use_ampm {
                            if hour == 0 {
                                hour = 12;
                            } else if hour > 12 {
                                hour -= 12;
                            }
                        }
                        text = format!("{text}{hour:02}");
                    }
                    TextToken::Second => {
                        let second = seconds;
                        text = format!("{text}{second}");
                    }
                    TextToken::SecondPadded => {
                        let second = seconds;
                        text = format!("{text}{second:02}");
                    }
                    TextToken::SecondDecimals(count) => {
                        let shown = units.rem_euclid(units_per_second)
                            / 10_i64.pow((decimals - count) as u32);
                        text = format!("{text}.{shown:0count$}");
                    }
                    TextToken::AMPM => {
                        let ampm = if hours < 12 { "AM" } else { "PM" };
                        text = format!("{text}{ampm}");
                    }
                    TextToken::AP(lower) => {
                        let ap = match (hours < 12, lower) {
                            (true, false) => "A",
                            (false, false) => "P",
                            (true, true) => "a",
                            (false, true) => "p",
                        };
                        text = format!("{text}{ap}");
                    }
                    TextToken::Minute => {
                        let minute = minutes;
                        text = format!("{text}{minute}");
                    }
                    TextToken::MinutePadded => {
                        let minute = minutes;
                        text = format!("{text}{minute:02}");
                    }
                    TextToken::ElapsedHour => {
                        let hour = elapsed_seconds / 3600;
                        text = format!("{text}{hour}");
                    }
                    TextToken::ElapsedHourPadded => {
                        let hour = elapsed_seconds / 3600;
                        text = format!("{text}{hour:02}");
                    }
                    TextToken::ElapsedMinute => {
                        let minute = elapsed_seconds / 60;
                        text = format!("{text}{minute}");
                    }
                    TextToken::ElapsedMinutePadded => {
                        let minute = elapsed_seconds / 60;
                        text = format!("{text}{minute:02}");
                    }
                    TextToken::ElapsedSecond => {
                        let second = elapsed_seconds;
                        text = format!("{text}{second}");
                    }
                    TextToken::ElapsedSecondPadded => {
                        let second = elapsed_seconds;
                        text = format!("{text}{second:02}");
                    }
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
        ParsePart::Number(p) => {
            let mut text = "".to_string();
            if let Some(c) = p.currency {
                text = format!("{c}");
            }
            let tokens = &p.tokens;
            value = value * 100.0_f64.powi(p.percent) / (1000.0_f64.powi(p.comma));
            // p.precision is the number of significant digits _after_ the decimal point
            value = to_precision(
                value,
                (p.precision as usize) + format!("{}", value.abs().floor()).len(),
            );
            let mut value_abs = value.abs();
            let mut exponent_part: Vec<char> = vec![];
            let mut exponent_is_negative = value_abs < 10.0;
            if p.is_scientific {
                if value_abs == 0.0 {
                    exponent_part = vec!['0'];
                    exponent_is_negative = false;
                } else {
                    // TODO: Implement engineering formatting.
                    let exponent = value_abs.log10().floor();
                    exponent_part = format!("{}", exponent.abs()).chars().collect();
                    value /= 10.0_f64.powf(exponent);
                    value = to_precision(value, 15);
                    value_abs = value.abs();
                }
            }
            let l_exp = exponent_part.len() as i32;
            let int_number = if p.precision == 0 {
                value_abs.round()
            } else {
                value_abs.floor()
            };
            let mut int_part: Vec<char> = format!("{}", int_number).chars().collect();
            if int_number as i64 == 0 {
                int_part = vec![];
            }
            let fract_part = get_fract_part(value_abs, p.precision, int_part.len());
            // ln is the number of digits of the integer part of the value
            let ln = int_part.len() as i32;
            // digit count is the number of digit tokens ('0', '?' and '#') to the left of the decimal point
            let digit_count = p.digit_count;
            // digit_index points to the digit index in value that we have already formatted
            let mut digit_index = 0;

            let symbols = &locale.numbers.symbols;
            let group_sizes = locale.numbers.decimal_formats.standard.to_owned();
            let group_separator = symbols.group.to_owned();
            let decimal_separator = symbols.decimal.to_owned();
            // There probably are better ways to check if a number at a given precision is negative :/
            let is_negative = value < -(10.0_f64.powf(-(p.precision as f64)));
            let mut needs_period = false;

            for token in tokens {
                match token {
                    TextToken::Literal(c) => {
                        text = format!("{text}{c}");
                    }
                    TextToken::Text(t) => {
                        text = format!("{text}{t}");
                    }
                    TextToken::Ghost(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Spacer(_) => {
                        // we just leave a whitespace
                        // This is what the TEXT function does
                        text = format!("{text} ");
                    }
                    TextToken::Raw => {
                        text = format!("{text}{value}");
                    }
                    TextToken::Period => {
                        // if !fract_part.is_empty() &&  {
                        //     text = format!("{text}{decimal_separator}");
                        // }
                        needs_period = true;
                    }
                    TextToken::Digit(digit) => {
                        if digit.number.is_integer() {
                            // 1. Integer part
                            let index = digit.index;
                            let number_index = ln - digit_count + index;
                            if index == 0 && is_negative {
                                text = format!("-{text}");
                            }
                            if ln <= digit_count {
                                // The number of digits is less or equal than the number of digit tokens
                                // i.e. the value is 123 and the format_code is ##### (ln = 3 and digit_count = 5)
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '0' {
                                            '0'
                                        } else {
                                            // digit.kind = '?'
                                            ' '
                                        }
                                    } else {
                                        int_part[number_index as usize]
                                    };
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        ln - digit_index,
                                        &group_sizes,
                                    ) {
                                        &group_separator
                                    } else {
                                        ""
                                    };
                                    text = format!("{text}{c}{sep}");
                                }
                                digit_index += 1;
                            } else {
                                // The number is larger than the formatting code 12345 and 0##
                                // We just hit the first formatting digit (0 in the example above) so we write as many digits as we can (123 in the example)
                                for i in digit_index..number_index + 1 {
                                    let sep = if use_group_separator(
                                        p.use_thousands,
                                        ln - i,
                                        &group_sizes,
                                    ) {
                                        &group_separator
                                    } else {
                                        ""
                                    };
                                    text = format!("{}{}{}", text, int_part[i as usize], sep);
                                }
                                digit_index = number_index + 1;
                            }
                        } else if digit.number.is_decimal() {
                            // 2. After the decimal point
                            let index = digit.index as usize;
                            if index < fract_part.len() {
                                if needs_period {
                                    text = format!(
                                        "{}{}{}",
                                        text, decimal_separator, fract_part[index]
                                    );
                                } else {
                                    text = format!("{}{}", text, fract_part[index]);
                                }
                            } else if digit.kind == '0' {
                                if needs_period {
                                    text = format!("{text}{}0", decimal_separator);
                                } else {
                                    text = format!("{text}0");
                                }
                            } else if digit.kind == '?' {
                                text = format!("{text} ");
                            } else if digit.kind == '#' && needs_period {
                                // FIXME: This is what Excel does, but it transforms "3" into "3."
                                text = format!("{text}{}", decimal_separator);
                            }
                            needs_period = false;
                        } else if digit.number.is_exponent() {
                            // 3. Exponent part
                            let index = digit.index;
                            if index == 0 {
                                if exponent_is_negative {
                                    text = format!("{text}E-");
                                } else if p.scientific_minus {
                                    text = format!("{text}E");
                                } else {
                                    text = format!("{text}E+");
                                }
                            }
                            let number_index = l_exp - (p.exponent_digit_count - index);
                            if l_exp <= p.exponent_digit_count {
                                if !(number_index < 0 && digit.kind == '#') {
                                    let c = if number_index < 0 {
                                        if digit.kind == '?' {
                                            ' '
                                        } else {
                                            '0'
                                        }
                                    } else {
                                        exponent_part[number_index as usize]
                                    };

                                    text = format!("{text}{c}");
                                }
                            } else {
                                for i in 0..number_index + 1 {
                                    text = format!("{}{}", text, exponent_part[i as usize]);
                                }
                                digit_index += number_index + 1;
                            }
                        }
                    }
                    // Date tokens should not be present
                    TextToken::Day => {}
                    TextToken::DayPadded => {}
                    TextToken::DayNameShort => {}
                    TextToken::DayName => {}
                    TextToken::Month => {}
                    TextToken::MonthPadded => {}
                    TextToken::MonthNameShort => {}
                    TextToken::MonthName => {}
                    TextToken::MonthLetter => {}
                    TextToken::YearShort => {}
                    TextToken::Year => {}
                    TextToken::Hour => {}
                    TextToken::HourPadded => {}
                    TextToken::Minute => {}
                    TextToken::MinutePadded => {}
                    TextToken::Second => {}
                    TextToken::SecondPadded => {}
                    TextToken::AMPM => {}
                    TextToken::AP(_) => {}
                    TextToken::ElapsedHour => {}
                    TextToken::ElapsedHourPadded => {}
                    TextToken::ElapsedMinute => {}
                    TextToken::ElapsedMinutePadded => {}
                    TextToken::ElapsedSecond => {}
                    TextToken::ElapsedSecondPadded => {}
                    TextToken::SecondDecimals(_) => {}
                }
            }
            Formatted {
                text,
                color: p.color,
                error: None,
            }
        }
    }
}

fn parse_day(day_str: &str) -> Result<(u32, String), String> {
    let bytes = day_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len <= 2 {
        match day_str.parse::<u32>() {
            Ok(y) => {
                if bytes_len == 2 {
                    return Ok((y, "dd".to_string()));
                } else {
                    return Ok((y, "d".to_string()));
                }
            }
            Err(_) => return Err("Not a valid day".to_string()),
        }
    }
    Err("Not a valid day".to_string())
}

fn parse_month(month_str: &str, locale: &Locale) -> Result<(u32, String), String> {
    let bytes = month_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len <= 2 {
        match month_str.parse::<u32>() {
            Ok(y) => {
                if bytes_len == 2 {
                    return Ok((y, "mm".to_string()));
                } else {
                    return Ok((y, "m".to_string()));
                }
            }
            Err(_) => return Err("Not a valid month".to_string()),
        }
    }

    let month_names_short = &locale.dates.months_short;
    let month_names_long = &locale.dates.months;
    if let Some(m) = month_names_short.iter().position(|r| r == month_str) {
        return Ok((m as u32 + 1, "mmm".to_string()));
    }
    if let Some(m) = month_names_long.iter().position(|r| r == month_str) {
        return Ok((m as u32 + 1, "mmmm".to_string()));
    }
    Err("Not a valid month".to_string())
}

fn parse_year(year_str: &str) -> Result<(i32, String), String> {
    // year is either 2 digits or 4 digits
    // 23 -> 2023
    // 75 -> 1975
    // 30 is the split number (yeah, that's not going to be a problem any time soon)
    // 30 => 1930
    // 29 => 2029
    let bytes = year_str.bytes();
    let bytes_len = bytes.len();
    if bytes_len != 2 && bytes_len != 4 {
        return Err("Not a valid year".to_string());
    }
    match year_str.parse::<i32>() {
        Ok(y) => {
            if y < 30 {
                Ok((2000 + y, "yy".to_string()))
            } else if y < 100 {
                Ok((1900 + y, "yy".to_string()))
            } else {
                Ok((y, "yyyy".to_string()))
            }
        }
        Err(_) => Err("Not a valid year".to_string()),
    }
}

// Check if it is a date. Other spreadsheet engines support a wide variety of dates formats
// Here we support a small subset of them.
//
// The grammar is:
//
// date -> long_date | short_date | iso-date
// short_date -> month separator year
// long_date -> day separator month separator year
// iso_date -> long_year separator number_month separator number_day
// separator -> "/" | "-" | "."
// day -> number | padded number
// month -> number_month | name_month
// number_month -> number | padded number |
// name_month -> short name | full name
// year -> short_year | long year
//
// NOTE 1: The separator has to be the same
// NOTE 2: In some engines "2/3" is implemented ad "2/March of the present year"
// NOTE 3: I did not implement the "short date"
pub(crate) fn parse_date(
    value: &str,
    locale: &Locale,
    date_system: DateSystem,
) -> Result<(i32, String), String> {
    let separator = if value.contains('/') {
        '/'
    } else if value.contains('-') {
        '-'
    } else if value.contains('.') {
        '.'
    } else {
        return Err("Not a valid date".to_string());
    };

    let parts: Vec<&str> = value.split(separator).collect();
    let mut is_iso_date = false;
    let mut day_first = true;
    let (day_str, month_str, year_str) = if parts.len() == 3 {
        if parts[0].len() == 4 {
            // ISO date  yyyy-mm-dd
            if !parts[1].chars().all(char::is_numeric) {
                return Err("Not a valid date".to_string());
            }
            if !parts[2].chars().all(char::is_numeric) {
                return Err("Not a valid date".to_string());
            }
            is_iso_date = true;
            (parts[2], parts[1], parts[0])
        } else {
            //  localized date dd-mm-yyyy or mm-dd-yyyy
            // TODO: A bit hacky, but works for now
            if locale.dates.date_formats.short.starts_with('d') {
                (parts[0], parts[1], parts[2])
            } else {
                day_first = false;
                (parts[1], parts[0], parts[2])
            }
        }
    } else {
        return Err("Not a valid date".to_string());
    };
    let (day, day_format) = parse_day(day_str)?;
    let (month, month_format) = parse_month(month_str, locale)?;
    let (year, year_format) = parse_year(year_str)?;
    let serial_number = match date_to_serial_number(day, month, year, date_system) {
        Ok(n) => n,
        Err(_) => return Err("Not a valid date".to_string()),
    };
    if is_iso_date {
        Ok((
            serial_number,
            format!("yyyy{separator}{month_format}{separator}{day_format}"),
        ))
    } else if !day_first {
        Ok((
            serial_number,
            format!("{month_format}{separator}{day_format}{separator}{year_format}"),
        ))
    } else {
        Ok((
            serial_number,
            format!("{day_format}{separator}{month_format}{separator}{year_format}"),
        ))
    }
}

/// IronXL: the value of a time component of 1 or 2 digits from 0 to `maximum`.
fn parse_time_component(text: &str, maximum: u32) -> Option<u32> {
    if text.is_empty() || text.len() > 2 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse::<u32>().ok().filter(|v| *v <= maximum)
}

/// IronXL: the milliseconds of 1 to 3 digits after the seconds, such as `25` in `12:00:00.25`.
fn parse_time_milliseconds(digits: &str) -> Option<u32> {
    if digits.is_empty() || digits.len() > 3 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let value = digits.parse::<u32>().ok()?;
    Some(value * 10_u32.pow(3 - digits.len() as u32))
}

/// IronXL: reads a time text as Excel does, and gives the fraction of a day and a time format.
///
/// The grammar is:
///
/// time -> hour ":" minute [":" second ["." digits]] [space] [am_pm]
/// hour -> 0 to 23, or 1 to 12 with am_pm; 1 or 2 digits
/// minute, second -> 0 to 59; 1 or 2 digits
/// digits -> 1 to 3 digits after the seconds
/// am_pm -> "AM" | "PM", in any case
///
/// TIMEVALUE and the text-to-number cast (VALUE, an operator, a number argument) use this parser.
pub(crate) fn parse_time(value: &str) -> Result<(f64, String), String> {
    let error = || "Not a valid time".to_string();
    let value = value.trim();
    let upper = value.to_ascii_uppercase();
    let (clock, is_pm) = if let Some(clock) = upper.strip_suffix("AM") {
        (clock.trim_end(), Some(false))
    } else if let Some(clock) = upper.strip_suffix("PM") {
        (clock.trim_end(), Some(true))
    } else {
        (upper.as_str(), None)
    };
    let parts: Vec<&str> = clock.split(':').collect();
    let (hour, minute, second) = match parts.as_slice() {
        [hour, minute] => (*hour, *minute, None),
        [hour, minute, second] => (*hour, *minute, Some(*second)),
        _ => return Err(error()),
    };
    let (second, digits) = match second {
        Some(second) => match second.split_once('.') {
            Some((second, digits)) => (second, Some(digits)),
            None => (second, None),
        },
        None => ("0", None),
    };
    let hour = parse_time_component(hour, 23).ok_or_else(error)?;
    let minute = parse_time_component(minute, 59).ok_or_else(error)?;
    let second = parse_time_component(second, 59).ok_or_else(error)?;
    let milliseconds = match digits {
        Some(digits) => parse_time_milliseconds(digits).ok_or_else(error)?,
        None => 0,
    };
    let hour = match is_pm {
        Some(_) if !(1..=12).contains(&hour) => return Err(error()),
        Some(is_pm) => hour % 12 + if is_pm { 12 } else { 0 },
        None => hour,
    };
    let total_milliseconds = ((hour * 60 + minute) * 60 + second) * 1000 + milliseconds;
    let mut format = match (parts.len(), digits) {
        (2, _) => "h:mm".to_string(),
        (_, Some(digits)) => format!("h:mm:ss.{}", "0".repeat(digits.len())),
        _ => "h:mm:ss".to_string(),
    };
    if is_pm.is_some() {
        format.push_str(" AM/PM");
    }
    Ok((total_milliseconds as f64 / 86_400_000.0, format))
}

/// IronXL: reads a date text, then white space, then a time text, as Excel does, such as
/// `1/2/2020 12:00`. Gives the date serial number plus the fraction of the day, and the date
/// format then the time format.
///
/// `parse_date` reads the date part and `parse_time` reads the time part.
pub(crate) fn parse_date_time(
    value: &str,
    locale: &Locale,
    date_system: DateSystem,
) -> Result<(f64, String), String> {
    let error = || "Not a valid date and time".to_string();
    let (date, time) = value
        .trim()
        .split_once(char::is_whitespace)
        .ok_or_else(error)?;
    let (serial_number, date_format) = parse_date(date, locale, date_system)?;
    let (fraction, time_format) = parse_time(time)?;
    Ok((
        serial_number as f64 + fraction,
        format!("{date_format} {time_format}"),
    ))
}

/// Parses a formatted number, returning the numeric value together with the format
/// Uses heuristics to guess the format string
/// "$ 123,345.678" => (123345.678, "$#,##0.00")
/// "30.34%" => (0.3034, "0.00%")
/// 100€ => (100, "100€")
#[cfg(test)]
pub(crate) fn parse_formatted_number(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
) -> Result<(f64, Option<String>), String> {
    parse_formatted_number_with_date_system(original, currencies, locale, DateSystem::Date1900)
}

/// IronXL: parses a number like `parse_formatted_number`. A date gives a serial number of
/// `date_system`.
pub(crate) fn parse_formatted_number_with_date_system(
    original: &str,
    currencies: &[&str],
    locale: &Locale,
    date_system: DateSystem,
) -> Result<(f64, Option<String>), String> {
    let value = original.trim();
    let scientific_format = "0.00E+00";

    let symbols = &locale.numbers.symbols;
    let decimal_separator = symbols.decimal.chars().next().unwrap_or('.');
    let group_separator = symbols.group.chars().next().unwrap_or(',');

    // Check if it is a percentage
    if let Some(p) = value.strip_suffix('%') {
        let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
        if options.is_scientific {
            return Ok((f / 100.0, Some(scientific_format.to_string())));
        }
        // We ignore the separator
        if options.decimal_digits > 0 {
            // Percentage format with decimals
            return Ok((f / 100.0, Some("#,##0.00%".to_string())));
        }
        // Percentage format standard
        return Ok((f / 100.0, Some("#,##0%".to_string())));
    }

    // check if it is a currency in currencies
    for currency in currencies {
        if let Some(p) = value.strip_prefix(&format!("-{currency}")) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                return Ok((-f, Some(format!("{currency}#,##0.00"))));
            }
            return Ok((-f, Some(format!("{currency}#,##0"))));
        } else if let Some(p) = value.strip_prefix(currency) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                return Ok((f, Some(format!("{currency}#,##0.00"))));
            }
            return Ok((f, Some(format!("{currency}#,##0"))));
        } else if let Some(p) = value.strip_suffix(currency) {
            let (f, options) = parse_number(p.trim(), decimal_separator, group_separator)?;
            if options.is_scientific {
                return Ok((f, Some(scientific_format.to_string())));
            }
            if options.decimal_digits > 0 {
                let currency_format = &format!("#,##0.00{currency}");
                return Ok((f, Some(currency_format.to_string())));
            }
            let currency_format = &format!("#,##0{currency}");
            return Ok((f, Some(currency_format.to_string())));
        }
    }

    // check if it is a date. NOTE: we don't trim the original here
    if let Ok((serial_number, format)) = parse_date(original, locale, date_system) {
        return Ok((serial_number as f64, Some(format)));
    }

    // IronXL: check if it is a time, such as 12:00:00.25
    if let Ok((fraction, format)) = parse_time(value) {
        return Ok((fraction, Some(format)));
    }

    // IronXL: check if it is a date and a time, such as 1/2/2020 12:00
    if let Ok((serial_number, format)) = parse_date_time(value, locale, date_system) {
        return Ok((serial_number, Some(format)));
    }

    // Lastly we check if it is a number
    let (f, options) = parse_number(value, decimal_separator, group_separator)?;
    if options.is_scientific {
        return Ok((f, Some(scientific_format.to_string())));
    }
    if options.has_commas {
        if options.decimal_digits > 0 {
            // group separator and two decimal points
            return Ok((f, Some("#,##0.00".to_string())));
        }
        // Group separator and no decimal points
        return Ok((f, Some("#,##0".to_string())));
    }
    Ok((f, None))
}

struct NumberOptions {
    has_commas: bool,
    is_scientific: bool,
    decimal_digits: usize,
}

// tries to parse 'value' as a number.
// If it is a number it either uses commas as thousands separator or it does not
fn parse_number(
    value: &str,
    decimal_separator: char,
    group_separator: char,
) -> Result<(f64, NumberOptions), String> {
    let mut position = 0;
    let characters: Vec<char> = value.chars().collect();
    let len = characters.len();
    if len == 0 {
        return Err("Cannot parse number".to_string());
    }
    let mut chars = String::from("");
    let mut group_separator_index = Vec::new();
    // get the sign
    let sign = if characters[0] == '-' {
        position += 1;
        -1.0
    } else if characters[0] == '+' {
        position += 1;
        1.0
    } else {
        1.0
    };

    if position >= len {
        return Err("Cannot parse number".to_string());
    }

    if characters[position] == group_separator {
        return Err("Cannot parse number".to_string());
    }

    // numbers before the decimal point
    while position < len {
        let x = characters[position];
        if x.is_ascii_digit() {
            chars.push(x);
        } else if x == group_separator {
            group_separator_index.push(chars.len());
        } else {
            break;
        }
        position += 1;
    }
    // Check the group separator is in multiples of three
    for index in &group_separator_index {
        if (chars.len() - index) % 3 != 0 {
            return Err("Cannot parse number".to_string());
        }
    }
    let mut decimal_digits = 0;
    if position < len && characters[position] == decimal_separator {
        // numbers after the decimal point
        chars.push('.');
        position += 1;
        let start_position = 0;
        while position < len {
            let x = characters[position];
            if x.is_ascii_digit() {
                chars.push(x);
            } else {
                break;
            }
            position += 1;
        }
        decimal_digits = position - start_position;
    }
    let mut is_scientific = false;
    if position + 1 < len && (characters[position] == 'e' || characters[position] == 'E') {
        // exponential side
        is_scientific = true;
        let x = characters[position + 1];
        if x == '-' || x == '+' || x.is_ascii_digit() {
            chars.push('e');
            chars.push(x);
            position += 2;
            while position < len {
                let x = characters[position];
                if x.is_ascii_digit() {
                    chars.push(x);
                } else {
                    break;
                }
                position += 1;
            }
        }
    }
    if position != len {
        return Err("Could not parse number".to_string());
    };
    match chars.parse::<f64>() {
        Err(_) => Err("Failed to parse to double".to_string()),
        Ok(v) => Ok((
            sign * v,
            NumberOptions {
                has_commas: !group_separator_index.is_empty(),
                is_scientific,
                decimal_digits,
            },
        )),
    }
}
