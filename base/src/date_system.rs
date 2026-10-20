//! The date system of the model that the engine evaluates (IronXL).
//!
//! Excel has two date systems. The 1900 system gives serial `1` to 1 January 1900 and counts the
//! fictional 29 February 1900. The 1904 system gives serial `0` to 1 January 1904. A workbook selects
//! the 1904 system with `workbookPr/@date1904`.
//!
//! Every date function and the number formatter read the serial base, the first serial, and the last
//! serial from this module. The model selects its date system for the length of each public call that
//! can read a date, through [`DateSystemScope`], so the free functions of the formatter need no model.

use std::cell::Cell;

use crate::constants::{EXCEL_DATE_BASE, MAXIMUM_DATE_SERIAL_NUMBER, MINIMUM_DATE_SERIAL_NUMBER};

/// Days from 1 January 1900 (serial 1462 in the 1900 system) to 1 January 1904 (serial 0 in the 1904
/// system).
pub(crate) const DATE1904_OFFSET: i32 = 1462;

thread_local! {
    static DATE1904: Cell<bool> = const { Cell::new(false) };
}

/// True while the current call evaluates a model in the 1904 date system.
pub(crate) fn is_date1904() -> bool {
    DATE1904.with(Cell::get)
}

/// The `num_days_from_ce` of serial `0` in the current date system.
pub(crate) fn excel_date_base() -> i32 {
    if is_date1904() {
        EXCEL_DATE_BASE + DATE1904_OFFSET
    } else {
        EXCEL_DATE_BASE
    }
}

/// The first serial of a date: 1 January 1900 in the 1900 system, 1 January 1904 in the 1904 system.
pub(crate) fn minimum_date_serial_number() -> i32 {
    if is_date1904() {
        0
    } else {
        MINIMUM_DATE_SERIAL_NUMBER
    }
}

/// The serial of 31 December 9999 in the current date system.
pub(crate) fn maximum_date_serial_number() -> i32 {
    if is_date1904() {
        MAXIMUM_DATE_SERIAL_NUMBER - DATE1904_OFFSET
    } else {
        MAXIMUM_DATE_SERIAL_NUMBER
    }
}

/// Selects a date system until the value drops, then selects the earlier one again.
pub(crate) struct DateSystemScope {
    earlier: bool,
}

impl DateSystemScope {
    /// Selects the 1904 date system when `date1904` is true, else the 1900 date system.
    pub(crate) fn enter(date1904: bool) -> Self {
        let earlier = DATE1904.with(|current| current.replace(date1904));
        DateSystemScope { earlier }
    }
}

impl Drop for DateSystemScope {
    fn drop(&mut self) {
        DATE1904.with(|current| current.set(self.earlier));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_1900_system_is_the_default() {
        assert!(!is_date1904());
        assert_eq!(excel_date_base(), EXCEL_DATE_BASE);
        assert_eq!(minimum_date_serial_number(), 1);
        assert_eq!(maximum_date_serial_number(), 2_958_465);
    }

    #[test]
    fn a_scope_selects_the_1904_system_and_restores_the_earlier_one() {
        {
            let _outer = DateSystemScope::enter(true);
            assert!(is_date1904());
            assert_eq!(excel_date_base(), EXCEL_DATE_BASE + 1462);
            assert_eq!(minimum_date_serial_number(), 0);
            assert_eq!(maximum_date_serial_number(), 2_957_003);
            {
                let _inner = DateSystemScope::enter(false);
                assert!(!is_date1904());
            }
            assert!(is_date1904());
        }
        assert!(!is_date1904());
    }
}
