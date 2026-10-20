use chrono::Datelike;
use chrono::Days;
use chrono::Months;
use chrono::NaiveDate;

use crate::constants::DATE_1904_OFFSET;
use crate::constants::EXCEL_DATE_BASE;
use crate::constants::MAXIMUM_DATE_SERIAL_NUMBER;
use crate::constants::MINIMUM_DATE_SERIAL_NUMBER;

pub const DATE_OUT_OF_RANGE_MESSAGE: &str = "Out of range parameters for date";

/// IronXL: the date system of a workbook. It sets the date of serial number 0.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DateSystem {
    /// Serial 1 is 1 January 1900, with Excel's fictional 29 February 1900.
    #[default]
    Date1900,
    /// Serial 0 is 1 January 1904. The 1900 leap-year bug does not apply.
    Date1904,
}

impl DateSystem {
    /// Days from the common era of the date of serial number 0.
    #[inline]
    pub(crate) fn base(self) -> i32 {
        match self {
            DateSystem::Date1900 => EXCEL_DATE_BASE,
            DateSystem::Date1904 => EXCEL_DATE_BASE + DATE_1904_OFFSET,
        }
    }

    /// The smallest serial number that is a valid date.
    #[inline]
    pub(crate) fn minimum_serial(self) -> i32 {
        match self {
            DateSystem::Date1900 => MINIMUM_DATE_SERIAL_NUMBER,
            DateSystem::Date1904 => 0,
        }
    }

    /// Converts a serial number of the 1900 date system to this date system.
    #[inline]
    pub(crate) fn serial_from_1900(self, serial: f64) -> f64 {
        serial - (self.base() - EXCEL_DATE_BASE) as f64
    }

    /// Converts a serial number of this date system to the 1900 date system. A negative
    /// serial number stays negative, so it stays out of range.
    #[inline]
    pub(crate) fn serial_to_1900(self, serial: i64) -> i64 {
        if serial < 0 {
            serial
        } else {
            serial + (self.base() - EXCEL_DATE_BASE) as i64
        }
    }

    /// The serial number of 9999-12-31.
    #[inline]
    pub(crate) fn maximum_serial(self) -> i32 {
        match self {
            DateSystem::Date1900 => MAXIMUM_DATE_SERIAL_NUMBER,
            DateSystem::Date1904 => MAXIMUM_DATE_SERIAL_NUMBER - DATE_1904_OFFSET,
        }
    }
}

/// Converts a date to its serial number in the date system.
#[inline]
pub fn serial_from_date(date: NaiveDate, date_system: DateSystem) -> i32 {
    date.num_days_from_ce() - date_system.base()
}

fn is_date_within_range(date: NaiveDate, date_system: DateSystem) -> bool {
    let serial = serial_from_date(date, date_system);
    serial >= date_system.minimum_serial() && serial <= date_system.maximum_serial()
}

pub fn from_excel_date(days: i64, date_system: DateSystem) -> Result<NaiveDate, String> {
    let minimum = date_system.minimum_serial();
    let maximum = date_system.maximum_serial();
    if days < minimum as i64 {
        return Err(format!("Excel date must be greater than {minimum}"));
    };
    if days > maximum as i64 {
        return Err(format!("Excel date must be less than {maximum}"));
    };
    #[allow(clippy::expect_used)]
    let dt = NaiveDate::from_num_days_from_ce_opt(date_system.base() + days as i32)
        .expect("problem with chrono::NaiveDate");
    Ok(dt)
}

pub fn date_to_serial_number(
    day: u32,
    month: u32,
    year: i32,
    date_system: DateSystem,
) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => Ok(serial_from_date(native_date, date_system)),
        None => Err(DATE_OUT_OF_RANGE_MESSAGE.to_string()),
    }
}

pub fn permissive_date_to_serial_number(
    day: i32,
    month: i32,
    year: i32,
    date_system: DateSystem,
) -> Result<i32, String> {
    // Excel parses `DATE` very permissively. It allows not just for valid date values, but it
    // allows for invalid dates as well. If you for example enter `DATE(1900, 1, 32)` it will
    // return the date `1900-02-01`. Despite giving a day that is out of range it will just
    // wrap the month and year around.
    //
    // This function applies that same logic to dates. And does it in the most compatible way as
    // possible.

    // Special case for the minimum date
    if date_system == DateSystem::Date1900 && year == 1899 && month == 12 && day == 31 {
        return Ok(MINIMUM_DATE_SERIAL_NUMBER);
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
    if !is_date_within_range(date, date_system) {
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
        if !is_date_within_range(date, date_system) {
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
        if !is_date_within_range(date, date_system) {
            return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
        }
        date
    };

    Ok(serial_from_date(date, date_system))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_permissive_date_to_serial_number() {
        assert_eq!(
            permissive_date_to_serial_number(42, 42, 2002, DateSystem::Date1900),
            date_to_serial_number(12, 7, 2005, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 42, 2002, DateSystem::Date1900),
            date_to_serial_number(1, 6, 2005, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 15, 2000, DateSystem::Date1900),
            date_to_serial_number(1, 3, 2001, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000, DateSystem::Date1900),
            date_to_serial_number(1, 1, 2004, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(1, 49, 2000, DateSystem::Date1900),
            date_to_serial_number(1, 1, 2004, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 49, 2000, DateSystem::Date1900),
            date_to_serial_number(31, 1, 2004, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 49, 2000, DateSystem::Date1900),
            date_to_serial_number(12, 9, 2004, DateSystem::Date1900)
        );
        assert_eq!(
            permissive_date_to_serial_number(256, 1, 2004, DateSystem::Date1900),
            date_to_serial_number(12, 9, 2004, DateSystem::Date1900)
        );
    }

    #[test]
    fn test_max_and_min_dates() {
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 9999, DateSystem::Date1900),
            Ok(MAXIMUM_DATE_SERIAL_NUMBER),
        );
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 1899, DateSystem::Date1900),
            Ok(MINIMUM_DATE_SERIAL_NUMBER),
        );
    }

    #[test]
    fn test_date_1904() {
        let date_1904 = DateSystem::Date1904;
        assert_eq!(date_to_serial_number(1, 1, 1904, date_1904), Ok(0));
        assert_eq!(date_to_serial_number(1, 1, 2026, date_1904), Ok(44_561));
        assert_eq!(
            from_excel_date(0, date_1904),
            Ok(NaiveDate::from_ymd_opt(1904, 1, 1).unwrap())
        );
        assert!(from_excel_date(-1, date_1904).is_err());
        assert!(from_excel_date(date_1904.maximum_serial() as i64 + 1, date_1904).is_err());
        assert_eq!(
            permissive_date_to_serial_number(31, 12, 9999, date_1904),
            Ok(date_1904.maximum_serial())
        );
        assert!(permissive_date_to_serial_number(31, 12, 1899, date_1904).is_err());
        assert!(permissive_date_to_serial_number(31, 12, 1903, date_1904).is_err());
        assert_eq!(date_1904.serial_to_1900(0), 1_462);
        assert_eq!(date_1904.serial_to_1900(-1), -1);
        assert_eq!(date_1904.serial_from_1900(1_462.5), 0.5);
        assert_eq!(DateSystem::Date1900.serial_to_1900(42), 42);
        assert_eq!(DateSystem::Date1900.serial_from_1900(42.0), 42.0);
    }
}
