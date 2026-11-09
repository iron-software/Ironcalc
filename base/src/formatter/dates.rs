use chrono::Datelike;
use chrono::Days;
use chrono::Months;
use chrono::NaiveDate;
use chrono::Weekday;

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

/// IronXL: the 1900 serial of the fictional 29 February 1900. Excel counts this day, so a date
/// before 1 March 1900 has a serial one lower than a day count from 30 December 1899 gives.
const FEBRUARY_29_1900: i64 = 60;

/// Converts a date to its serial number in the date system.
///
/// IronXL: in the 1900 date system, a date before 1 March 1900 counts the days from
/// 31 December 1899, as Excel does (IronXL ADR 029).
#[inline]
pub fn serial_from_date(date: NaiveDate, date_system: DateSystem) -> i32 {
    let days = date.num_days_from_ce() - date_system.base();
    if date_system == DateSystem::Date1900 && (days as i64) <= FEBRUARY_29_1900 {
        days - 1
    } else {
        days
    }
}

/// IronXL: the number of days from `start` to `end`, as Excel counts them. In the 1900 date
/// system, Excel counts the fictional 29 February 1900. So the count from a date before
/// 1 March 1900 to a later date is one more than the calendar count (IronXL ADR 029).
pub fn days_between(start: NaiveDate, end: NaiveDate, date_system: DateSystem) -> i64 {
    serial_from_date(end, date_system) as i64 - serial_from_date(start, date_system) as i64
}

/// IronXL: the date of a serial number, with no range check. In the 1900 date system, a serial
/// below 60 counts the days from 31 December 1899, and serial 60, the fictional 29 February 1900,
/// is 28 February 1900. So serial 0 is 31 December 1899 and serial 1 is 1 January 1900
/// (IronXL ADR 029).
pub(crate) fn date_of_serial(days: i64, date_system: DateSystem) -> Option<NaiveDate> {
    let days = if date_system == DateSystem::Date1900 && days <= FEBRUARY_29_1900 {
        (days + 1).min(FEBRUARY_29_1900)
    } else {
        days
    };
    let days = i32::try_from(days).ok()?.checked_add(date_system.base())?;
    NaiveDate::from_num_days_from_ce_opt(days)
}

/// IronXL: the weekday of a serial number, as Excel gives it. Excel counts the fictional
/// 29 February 1900, so in the 1900 date system serial 0 is a Saturday and serial 1 a Sunday.
/// From serial 61, and in the 1904 date system, the weekday is the weekday of the date.
pub fn weekday_of_serial(days: i64, date_system: DateSystem) -> Weekday {
    const WEEKDAYS: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];
    // Day 1 of the common era, 1 January of year 1, is a Monday.
    let days_from_monday = (date_system.base() as i64 - 1 + days.rem_euclid(7)).rem_euclid(7);
    WEEKDAYS[days_from_monday as usize]
}

/// IronXL: the ISO week number of a serial number, with the weekday of Excel. The week holds
/// the Thursday that gives the year of the week.
pub fn iso_week_of_serial(days: i64, date_system: DateSystem) -> Option<u32> {
    let monday = days - weekday_of_serial(days, date_system).num_days_from_monday() as i64;
    let thursday = monday + 3;
    let year = date_of_serial(thursday, date_system)?.year();
    let first_day = serial_from_date(NaiveDate::from_ymd_opt(year, 1, 1)?, date_system) as i64;
    Some(((thursday - first_day) / 7 + 1) as u32)
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
    let dt = date_of_serial(days, date_system).expect("problem with chrono::NaiveDate");
    Ok(dt)
}

pub fn date_to_serial_number(
    day: u32,
    month: u32,
    year: i32,
    date_system: DateSystem,
) -> Result<i32, String> {
    match NaiveDate::from_ymd_opt(year, month, day) {
        Some(native_date) => {
            let serial = serial_from_date(native_date, date_system);
            // IronXL: the 1904 date system has no date before 1904-01-01. The 1900 date system
            // keeps the upstream behaviour.
            if date_system == DateSystem::Date1904 && serial < date_system.minimum_serial() {
                return Err(DATE_OUT_OF_RANGE_MESSAGE.to_string());
            }
            Ok(serial)
        }
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
        // IronXL: serial 1 is 1 January 1900 and 31 December 1899 is serial 0, below the
        // minimum (IronXL ADR 029).
        assert_eq!(
            permissive_date_to_serial_number(1, 1, 1900, DateSystem::Date1900),
            Ok(MINIMUM_DATE_SERIAL_NUMBER),
        );
        assert!(permissive_date_to_serial_number(31, 12, 1899, DateSystem::Date1900).is_err());
    }

    // IronXL: the 1900 serials of Excel before 1 March 1900 (IronXL ADR 029).
    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_serials_before_march_1900() {
        let date_1900 = DateSystem::Date1900;
        let date = |y, m, d| NaiveDate::from_ymd_opt(y, m, d).unwrap();
        assert_eq!(serial_from_date(date(1899, 12, 31), date_1900), 0);
        assert_eq!(serial_from_date(date(1900, 1, 1), date_1900), 1);
        assert_eq!(serial_from_date(date(1900, 2, 28), date_1900), 59);
        assert_eq!(serial_from_date(date(1900, 3, 1), date_1900), 61);
        assert_eq!(date_of_serial(-1, date_1900), Some(date(1899, 12, 30)));
        assert_eq!(date_of_serial(0, date_1900), Some(date(1899, 12, 31)));
        assert_eq!(date_of_serial(1, date_1900), Some(date(1900, 1, 1)));
        assert_eq!(date_of_serial(59, date_1900), Some(date(1900, 2, 28)));
        assert_eq!(date_of_serial(60, date_1900), Some(date(1900, 2, 28)));
        assert_eq!(date_of_serial(61, date_1900), Some(date(1900, 3, 1)));
        assert_eq!(date_of_serial(i64::MAX, date_1900), None);
        assert_eq!(from_excel_date(1, date_1900), Ok(date(1900, 1, 1)));
        assert!(from_excel_date(0, date_1900).is_err());
        assert_eq!(weekday_of_serial(0, date_1900), Weekday::Sat);
        assert_eq!(weekday_of_serial(1, date_1900), Weekday::Sun);
        assert_eq!(weekday_of_serial(59, date_1900), Weekday::Tue);
        assert_eq!(weekday_of_serial(60, date_1900), Weekday::Wed);
        assert_eq!(weekday_of_serial(61, date_1900), Weekday::Thu);
        assert_eq!(weekday_of_serial(-1, date_1900), Weekday::Fri);
        assert_eq!(iso_week_of_serial(1, date_1900), Some(52));
        assert_eq!(iso_week_of_serial(2, date_1900), Some(1));
        assert_eq!(iso_week_of_serial(i64::MAX - 7, date_1900), None);
        assert_eq!(weekday_of_serial(0, DateSystem::Date1904), Weekday::Fri);
        assert_eq!(iso_week_of_serial(0, DateSystem::Date1904), Some(53));
        assert_eq!(
            days_between(date(1900, 1, 1), date(1900, 3, 1), date_1900),
            60
        );
        assert_eq!(
            days_between(date(1900, 1, 1), date(1900, 2, 28), date_1900),
            58
        );
        assert_eq!(
            days_between(date(1900, 3, 1), date(1900, 1, 1), date_1900),
            -60
        );
        assert_eq!(
            days_between(date(2026, 1, 1), date(2026, 3, 1), date_1900),
            59
        );
        let date_1904 = DateSystem::Date1904;
        assert_eq!(
            days_between(date(1904, 1, 1), date(1904, 3, 1), date_1904),
            60
        );
    }

    #[test]
    fn test_date_1904() {
        let date_1904 = DateSystem::Date1904;
        assert_eq!(date_to_serial_number(1, 1, 1904, date_1904), Ok(0));
        assert_eq!(date_to_serial_number(1, 1, 2026, date_1904), Ok(44_561));
        assert_eq!(
            from_excel_date(0, date_1904).ok(),
            NaiveDate::from_ymd_opt(1904, 1, 1)
        );
        assert!(NaiveDate::from_ymd_opt(1904, 1, 1).is_some());
        assert!(date_to_serial_number(31, 12, 1903, date_1904).is_err());
        // IronXL: a date before 1 March 1900 counts the days from 31 December 1899 (IronXL ADR 029).
        assert_eq!(
            date_to_serial_number(31, 12, 1898, DateSystem::Date1900),
            Ok(-365)
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
