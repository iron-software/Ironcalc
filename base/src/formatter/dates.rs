use chrono::Datelike;
use chrono::Days;
use chrono::Duration;
use chrono::Months;
use chrono::NaiveDate;

use crate::date_system::excel_date_base;
use crate::date_system::is_date1904;
use crate::date_system::maximum_date_serial_number;
use crate::date_system::minimum_date_serial_number;

pub const DATE_OUT_OF_RANGE_MESSAGE: &str = "Out of range parameters for date";

#[inline]
fn convert_to_serial_number(date: NaiveDate) -> i32 {
    date.num_days_from_ce() - excel_date_base()
}

fn is_date_within_range(date: NaiveDate) -> bool {
    convert_to_serial_number(date) >= minimum_date_serial_number()
        && convert_to_serial_number(date) <= maximum_date_serial_number()
}

pub fn from_excel_date(days: i64) -> Result<NaiveDate, String> {
    if days < minimum_date_serial_number() as i64 {
        return Err(format!(
            "Excel date must be greater than {}",
            minimum_date_serial_number()
        ));
    };
    if days > maximum_date_serial_number() as i64 {
        return Err(format!(
            "Excel date must be less than {}",
            maximum_date_serial_number()
        ));
    };
    // IronXL: the base of the current date system. In the 1900 system it is two days before
    // 1 January 1900, because of the fictional 29 February 1900.
    #[allow(clippy::expect_used)]
    let base = NaiveDate::from_num_days_from_ce_opt(excel_date_base())
        .expect("problem with chrono::NaiveDate");
    Ok(base + Duration::days(days))
}

pub fn date_to_serial_number(day: u32, month: u32, year: i32) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => Ok(convert_to_serial_number(native_date)),
        None => Err(DATE_OUT_OF_RANGE_MESSAGE.to_string()),
    }
}

pub fn permissive_date_to_serial_number(day: i32, month: i32, year: i32) -> Result<i32, String> {
    // Excel parses `DATE` very permissively. It allows not just for valid date values, but it
    // allows for invalid dates as well. If you for example enter `DATE(1900, 1, 32)` it will
    // return the date `1900-02-01`. Despite giving a day that is out of range it will just
    // wrap the month and year around.
    //
    // This function applies that same logic to dates. And does it in the most compatible way as
    // possible.

    // Special case for the minimum date of the 1900 system
    if !is_date1904() && year == 1899 && month == 12 && day == 31 {
        return Ok(minimum_date_serial_number());
    }
    let Some(mut date) = NaiveDate::from_ymd_opt(year, 1, 1) else {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    };

    // One thing to note for example is that even if you started with a year out of range
    // but tried to increment the months so that it wraps around into within range, excel
    // would still return an error.
    //
    // I.E. DATE(0,13,-1) will return an error, despite it being equivalent to DATE(1,1,0) which
    // is within range.
    //
    // As a result, we have to run range checks as we parse the date from the biggest unit to the
    // smallest unit.
    if !is_date_within_range(date) {
        return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
    }

    date = {
        let month_diff = month - 1;
        let abs_month = month_diff.unsigned_abs();
        if month_diff <= 0 {
            date = date - Months::new(abs_month);
        } else {
            date = date + Months::new(abs_month);
        }
        if !is_date_within_range(date) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    date = {
        let day_diff = day - 1;
        let abs_day = day_diff.unsigned_abs() as u64;
        if day_diff <= 0 {
            date = date - Days::new(abs_day);
        } else {
            date = date + Days::new(abs_day);
        }
        if !is_date_within_range(date) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    Ok(convert_to_serial_number(date))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_date_to_serial_number() {
        assert_eq!(
            permissive_date_to_serial_number(42, 42, 2002),
            date_to_serial_number(12, 7, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 42, 2002),
            date_to_serial_number(1, 6, 2005)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 15, 2000),
            date_to_serial_number(1, 3, 2001)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000),
            date_to_serial_number(1, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 49, 2000),
            date_to_serial_number(31, 1, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 49, 2000),
            date_to_serial_number(12, 9, 2004)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 1, 2004),
            date_to_serial_number(12, 9, 2004)
        );
    }

    #[test]
    fn test_max_and_min_dates() {
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 9999),
            Ok(maximum_date_serial_number()),
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 1899),
            Ok(minimum_date_serial_number()),
        );
    }
}
